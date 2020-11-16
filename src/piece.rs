use crate::util::*;

pub struct Piece {
    pub color: Color,
    pub shape: Vec<Vec<u8>>,
}

impl Clone for Piece {
    fn clone(&self) -> Piece {
        let mut p = Piece{
            color: self.color,
            shape: Vec::with_capacity(self.shape.len())
        };
        for row in &self.shape {
            p.shape.push(row.clone());
        }
        p
    }
}

impl Piece {
    pub fn new_o() -> Piece {
        Piece{
            color: Color::Yellow,
            shape: vec![vec![1, 1],
                        vec![1, 1]]
        }
    }

    pub fn new_l() -> Piece {
        Piece{
            color: Color::Orange,
            shape: vec![vec![0, 0, 1],
                        vec![1, 1, 1],
                        vec![0, 0, 0]]
        }
    }

    pub fn new_j() -> Piece {
        Piece{
            color: Color::Blue,
            shape: vec![vec![1, 0, 0],
                        vec![1, 1, 1],
                        vec![0, 0, 0]]
        }
    }

    pub fn new_t() -> Piece {
        Piece{
            color: Color::Purple,
            shape: vec![vec![0, 1, 0],
                        vec![1, 1, 1],
                        vec![0, 0, 0]]
        }
    }

    pub fn new_s() -> Piece {
        Piece{
            color: Color::Green,
            shape: vec![vec![0, 1, 1],
                        vec![1, 1, 0],
                        vec![0, 0, 0]]
        }
    }

    pub fn new_z() -> Piece {
        Piece{
            color: Color::Red,
            shape: vec![vec![1, 1, 0],
                        vec![0, 1, 1],
                        vec![0, 0, 0]]
        }
    }

    pub fn new_i() -> Piece {
        Piece{
            color: Color::Cyan,
            shape: vec![vec![0, 0, 0, 0],
                        vec![1, 1, 1, 1],
                        vec![0, 0, 0, 0],
                        vec![0, 0, 0, 0]]
        }
    }

    pub fn rotate(&mut self, direction: Direction) {
        let size = self.shape.len();

        for row in 0..size/2 {
            for col in row..(size - row - 1) {
                let t = self.shape[row][col];

                match direction {
                    Direction::Left => {
                        self.shape[row][col] = self.shape[col][size - row - 1];
                        self.shape[col][size - row - 1] = self.shape[size - row - 1][size - col - 1];
                        self.shape[size - row - 1][size - col - 1] = self.shape[size - col - 1][row];
                        self.shape[size - col - 1][row] = t;
                    },
                    Direction::Right => {
                        self.shape[row][col] = self.shape[size - col - 1][row];
                        self.shape[size - col - 1][row] = self.shape[size - row - 1][size - col - 1];
                        self.shape[size - row - 1][size - col - 1] = self.shape[col][size - row - 1];
                        self.shape[col][size - row - 1] = t;
                    }
                }
            }
        }
    }

    pub fn each_point(&self, callback: &mut dyn FnMut(i32, i32)) {
        let piece_width = self.shape.len() as i32;
        for row in 0..piece_width {
            for col in 0..piece_width {
                if self.shape[row as usize][col as usize] != 0 {
                    callback(row, col);
                }
            }
        }
    }

    pub fn get_shadow_color(&self) -> Color
    {   
        match self.color
        {
            Color::Cyan => Color::DarkCyan,
            Color::Purple => Color::DarkPurple,
            Color::Green => Color::DarkGreen,
            Color::Red => Color::DarkRed,
            Color::Blue => Color::DarkBlue,
            Color::Orange => Color::DarkOrange,
            Color::Yellow => Color::DarkYellow,
            _ => Color::Black
        }
    }
}

/// Implements a queue of randomized tetrominoes.
///
/// Instead of a purely random stream of tetromino types, this queue generates a random ordering of all
/// possible types and ensures all of those pieces are used before re-generating a new random set. This helps
/// avoid pathological cases where purely random generation provides the same piece type repeately in a row,
/// or fails to provide a required piece for a very long time.
pub struct PieceBag {
    pieces: Vec<Piece>
}

impl PieceBag {
    pub fn new() -> PieceBag {
        let mut p = PieceBag{
            pieces: Vec::new()
        };
        p.fill_bag();
        p
    }

    /// Removes and returns the next piece in the queue.
    pub fn pop(&mut self) -> Piece {
        let piece = self.pieces.remove(0);
        if self.pieces.is_empty() {
            self.fill_bag();
        }
        piece
    }

    /// Returns a copy of the next piece in the queue.
    pub fn peek(&self) -> Piece {
        match self.pieces.first() {
            Some(p) => p.clone(),
            None => panic!("No next piece in piece bag")
        }
    }

    /// Generates a random ordering of all possible pieces and adds them to the piece queue.
    fn fill_bag(&mut self) {
        use rand::Rng;

        let mut pieces: Vec<Piece> = vec![
            Piece::new_o(),
            Piece::new_l(),
            Piece::new_j(),
            Piece::new_t(),
            Piece::new_s(),
            Piece::new_z(),
            Piece::new_i()
        ];

        let mut rng = rand::thread_rng();
        while !pieces.is_empty() {
            let i = rng.gen::<usize>() % pieces.len();
            self.pieces.push(pieces.swap_remove(i));
        }
    }
}