extern crate rand;

mod util;
mod display;
mod scores;
mod piece;


use piece::*;
use std::io::stdout;
use std::cell::RefCell;
use display::Display;
use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use util::*;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use clap::clap_app;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};

const BOARD_WIDTH: u32 = 10;
const BOARD_HEIGHT: u32 = 20;
const HIDDEN_ROWS: u32 = 2;

enum Key {
    Up,
    Down,
    Left,
    Right,
    Space,
    CtrlC,
    Hold,
    Pause,
    Char(char),
}

enum GameUpdate {
    KeyPress(Key),
    Tick,
}

#[derive(Debug, Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
}

struct Board {
    cells: [[Option<Color>; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
}

impl Board {
    pub fn render(&self, display: &mut Display) {
        for y in HIDDEN_ROWS..BOARD_HEIGHT {
            display.set_text("|", 0, y, Color::Red, Color::Black);
            display.set_text("|", BOARD_WIDTH * 2 + 1, y, Color::Red, Color::Black);
        }
        for x in 0..(BOARD_WIDTH * 2 + 1) {
            display.set_text("-", x, BOARD_HEIGHT, Color::Red, Color::Black);
        }
        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH {
                match self.cells[row as usize][col as usize] {
                    Some(color) => {
                        let c = 1 + (col * 2);
                        display.set_text(" ", c, row, color, color);
                        display.set_text(" ", c + 1, row, color, color);
                    },
                    None => ()
                }
            }
        }
    }

    pub fn lock_piece(&mut self, piece: &Piece, origin: Point) {
        piece.each_point(&mut |row, col| {
            let x = origin.x + (col as i32);
            let y = origin.y + (row as i32);
            self.cells[y as usize][x as usize] = Some(piece.color);
        });
    }

    pub fn collision_test(&self, piece: &Piece, origin: Point) -> bool {
        let mut found = false;
        piece.each_point(&mut |row, col| {
            if !found {
                let x = origin.x + col;
                let y = origin.y + row;
                if x < 0 || x >= (BOARD_WIDTH as i32) || y < 0 || y >= (BOARD_HEIGHT as i32) ||
                    self.cells[y as usize][x as usize] != None {
                  found = true;
                }
            }
        });

        found
    }

    /// Clears the board of any complete lines, shifting down rows to take their place.
    /// Returns the total number of lines that were cleared.
    fn clear_lines(&mut self) -> u32 {
        let mut cleared_lines: usize = 0;
        for row in (0..self.cells.len()).rev() {
            if (row as i32) - (cleared_lines as i32) < 0 {
                break;
            }

            if cleared_lines > 0 {
                self.cells[row] = self.cells[row - cleared_lines];
                self.cells[row - cleared_lines] = [None; BOARD_WIDTH as usize];
            }

            while !self.cells[row].iter().any(|x| *x == None) {
                cleared_lines += 1;
                self.cells[row] = self.cells[row - cleared_lines];
                self.cells[row - cleared_lines] = [None; BOARD_WIDTH as usize];
            }
        }

        cleared_lines as u32
    }
}





struct Game {
    board: Board,
    piece_bag: PieceBag,
    piece: Piece,
    hold: Option<Piece>,
    piece_position: Point,
    score: u32,
    switched: bool,
    level: u32,
    speed: Arc<AtomicU64>,
    to_clear: i32,
    paused: Arc<AtomicBool>
}

impl Game {
    fn new() -> Game {
        let mut piece_bag = PieceBag::new();
        let piece = piece_bag.pop();

        let mut game = Game {
            board: Board{
                cells: [[None; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize]
            },
            piece_bag: piece_bag,
            piece: piece,
            hold: None,
            piece_position: Point{ x: 0, y: 0 }, 
            score: 0,
            switched: false,
            level: 1,
            speed: Arc::new(AtomicU64::new(500)),
            to_clear: 10,
            paused: Arc::new(AtomicBool::new(false)),
        };

        game.place_new_piece();
        game
    }

    /// Returns the new position of the current piece if it were to be dropped.
    fn find_dropped_position(&self) -> Point {
        let mut origin = self.piece_position;
        while !self.board.collision_test(&self.piece, origin) {
            origin.y += 1;
        }
        origin.y -= 1;
        origin
    }

    /// Draws the game to the display.
    fn render(&self, display: &mut Display) {
        // Render the board
        self.board.render(display);

        // Render the level
        let left_margin = BOARD_WIDTH * 2 + 5;
        display.set_text(format!("Level: {}", self.level), left_margin, 3, Color::Red, Color::Black);

        //render score
        display.set_text(format!("Score: {}", self.score), left_margin, 5, Color::Red, Color::Black);

        // Render a ghost piece
        let x = 1 + (2 * self.piece_position.x);
        let ghost_position = self.find_dropped_position();
        self.render_piece(display, &self.piece, Point{ x: x, y: ghost_position.y }, true);

        // Render the currently falling piece
        self.render_piece(display, &self.piece, Point{ x: x, y: self.piece_position.y }, false);

        // Render the next piece
        display.set_text("Next piece:", left_margin, 7, Color::Red, Color::Black);
        let next_piece = self.piece_bag.peek();
        self.render_piece(display, &next_piece, Point{ x: (left_margin as i32) + 2, y: 9 }, false);

        //TODO Render hold piece
        display.set_text("Holding:", left_margin, 11, Color::Red, Color::Black);
        if let Some(p) = &self.hold
        {
            self.render_piece(display, &p, Point{ x: (left_margin as i32) + 2, y: 13 }, false);
        }
        
    }

    fn render_piece(&self, display: &mut Display, piece: &Piece, origin: Point, ghost: bool) {
        let color = piece.color;

        piece.each_point(&mut |row, col| {
            let x = (origin.x + 2 * col) as u32;
            let y = (origin.y + row) as u32;
            if ghost
            {
                display.set_text(" ", x, y, piece.get_shadow_color(), piece.get_shadow_color());
                display.set_text(" ", x + 1, y, piece.get_shadow_color(), piece.get_shadow_color());
            }
            else
            {
                display.set_text(" ", x, y, color, color);
                display.set_text(" ", x + 1, y, color, color);
            }
            
        });
    }

    /// Moves the current piece in the specified direction. Returns true if the piece could be moved and
    /// didn't collide.
    fn move_piece(&mut self, x: i32, y: i32) -> bool {
        let new_position = Point{
            x: self.piece_position.x + x,
            y: self.piece_position.y + y,
        };
        if self.board.collision_test(&self.piece, new_position) {
            false
        } else {
            self.piece_position = new_position;
            true
        }
    }

    /// Rotates the current piece in the specified direction. Returns true if the piece could be rotated
    /// without any collisions.
    fn rotate_piece(&mut self, direction: Direction) -> bool {
        let mut new_piece = self.piece.clone();
        new_piece.rotate(direction);

        if self.board.collision_test(&new_piece, self.piece_position) {
            false
        } else {
            self.piece = new_piece;
            true
        }
    }


    /// Switches the current piece with the held piece
    /// Places a new piece when hold was empty previously
    fn switch_hold(&mut self) -> bool
    {
        if self.switched
        {
            return false;   
        }
        if let Some(p) = &self.hold
        {
           let tmp = p.clone();
           self.hold = Some(self.piece.clone());
           self.piece = tmp;
        }
        else
        {
            self.hold = Some(self.piece.clone());
            self.piece = self.piece_bag.pop();
            
        }
        self.switched = true;
        return self.place_new_piece();
    }

    ///Pauses or unpauses the game
    fn pause(&self) -> bool
    {
        let p = self.paused.load(Ordering::SeqCst);
        self.paused.store(!p, Ordering::SeqCst);
        true
    }

    /// Positions the current piece at the top of the board. Returns true if the piece can be placed without
    /// any collisions.
    fn place_new_piece(&mut self) -> bool {
        let origin = Point{
            x: ((BOARD_WIDTH - (self.piece.shape.len() as u32)) / 2) as i32,
            y: 0,
        };
        if self.board.collision_test(&self.piece, origin) {
            false
        } else {
            self.piece_position = origin;
            true
        }
    }

    /// Advances the game by moving the current piece down one step. If the piece cannot move down, the piece
    /// is locked and the game is set up to drop the next piece.  Returns true if the game could be advanced,
    /// false if the player has lost.
    fn advance_game(&mut self) -> bool {
        if !self.move_piece(0, 1) {
            self.board.lock_piece(&self.piece, self.piece_position);
            let cleared = self.board.clear_lines();
            match cleared
            {
                1 => self.score += 100*self.level,
                2 => self.score += 300*self.level,
                3 => self.score += 500*self.level,
                4 => self.score += 800*self.level,
                _ => () 
            }
            self.to_clear -= cleared as i32;
            if self.to_clear <= 0
            {
                self.level += 1;
                self.to_clear = self.level as i32 * 10;
                let new_speed = num::clamp(500 - (self.level*20), 20, 500);

                self.speed.store(new_speed as u64, Ordering::SeqCst);
            }
            self.piece = self.piece_bag.pop();
            self.switched = false;
            if !self.place_new_piece() {
                return false;
            }
        }

        true
    }

    /// Drops the current piece to the lowest spot on the board where it fits without collisions and
    /// advances the game.
    fn drop_piece(&mut self) -> bool {
        while self.move_piece(0, 1) {}
        self.advance_game()
    }

    fn keypress(&mut self, key: Key) {
        if self.paused.load(Ordering::SeqCst)
        {
            match key {
                Key::Pause => self.pause(),
                _ => false,
            };
            return
        }
        match key {
            Key::Left => self.move_piece(-1, 0),
            Key::Right => self.move_piece(1, 0),
            Key::Down => self.advance_game(),
            Key::Up => self.rotate_piece(Direction::Left),
            Key::Space => self.drop_piece(),
            Key::Hold => self.switch_hold(),
            Key::Pause => self.pause(),
            Key::Char('q') => self.rotate_piece(Direction::Left),
            Key::Char('e') => self.rotate_piece(Direction::Right),
            _ => false,
        };
    }


    fn play(&mut self, display: &mut Display) {
        let (tx_event, rx_event) = mpsc::channel();
        // Spawn a thread which sends periodic game ticks to advance the piece
        {
            let tx_event = tx_event.clone();
            let arc = self.speed.clone();
            let p = self.paused.clone();
            thread::spawn(move || {
                loop {
                    let dur = Duration::from_millis(arc.load(Ordering::SeqCst));
                    thread::sleep(dur);
                    if !p.load(Ordering::SeqCst)
                    {
                        if let Ok(_) = tx_event.send(GameUpdate::Tick)
                        {}
                        else {break}
                    }
                };
            });
        }

        // Spawn a thread which listens for keyboard input
            let tx_event = tx_event.clone();
            let (flag, control) = thread_control::make_pair();

         let input_handle = thread::spawn(move || {
                let stdin = &mut std::io::stdin();

                while flag.alive() {
                    if let Ok(_) = match get_input(stdin) {
                        Some(k) => tx_event.send(GameUpdate::KeyPress(k)),
                        None => Ok(())
                    }{}
                    else{break}
                }
            });

        // Main game loop. The loop listens and responds to timer and keyboard updates received on a channel
        // as sent by the threads spawned above.
        loop {
            display.clear_buffer();
            self.render(display);
            display.render();

            match rx_event.recv() {
                Ok(update) => {
                    match update {
                        GameUpdate::KeyPress(key) => {
                            match key {
                                Key::Char('z') | Key::CtrlC => break,
                                k => { self.keypress(k); }
                            };
                        },
                        GameUpdate::Tick => { if !self.advance_game()
                                                {break} }
                    };
                },
                Err(err) => panic!(err)
            }
        }
        control.stop();
        input_handle.join().unwrap(); //to prevent input thread from eating input
    }
}

fn get_input(stdin: &mut std::io::Stdin) -> Option<Key> {
    use std::io::Read;

    let c = &mut [0u8];
    match stdin.read(c) {
        Ok(_) => {
            match std::str::from_utf8(c) {
                Ok("w") => Some(Key::Up),
                Ok("a") => Some(Key::Left),
                Ok("s") => Some(Key::Down),
                Ok("d") => Some(Key::Right),
                Ok(" ") => Some(Key::Space),
                Ok("c") => Some(Key::Hold),
                Ok("z") => Some(Key::CtrlC),
                Ok("p") => Some(Key::Pause),
                // Escape sequence started - must read two more bytes.
                Ok("\x1b") => {
                    let code = &mut [0u8; 2];
                    match stdin.read(code) {
                        Ok(_) => {
                            match std::str::from_utf8(code) {
                                Ok("[A") => Some(Key::Up),
                                Ok("[B") => Some(Key::Down),
                                Ok("[C") => Some(Key::Right),
                                Ok("[D") => Some(Key::Left),
                                _ => None
                            }
                        },
                        Err(msg) => panic!(format!("could not read from standard in: {}", msg))
                    }
                },
                Ok(n) => Some(Key::Char(n.chars().next().unwrap())),
                _ => None
            }
        },
        Err(msg) => panic!(format!("could not read from standard in: {}", msg))
    }
}

fn main() {
    let matches = clap_app!(Tetris =>
        (version: "1.0")
        (author: "royalmustard <royalmustard@memium.de>")
        (about: "Tetris (but its big stonks)")
        (@arg SCORES: -s --scores "Print highscores")
    ).get_matches();

    if matches.is_present("SCORES")
    {
        scores::print_highscores();
        return;
    }


    let (send, recv) = std::sync::mpsc::channel();
    
    let handle = std::thread::spawn(move || {
        let display = &mut Display::new(BOARD_WIDTH * 2 + 100, BOARD_HEIGHT + 2, 
            RefCell::new(Box::new(AlternateScreen::from(stdout().into_raw_mode().unwrap()))));
        let game = &mut Game::new();
        game.play(display);
        send.send(game.score).unwrap();
    });

    if let Ok(score) = recv.recv()
    {
        handle.join().unwrap();
        scores::manage_highscore(score);
    }
}
