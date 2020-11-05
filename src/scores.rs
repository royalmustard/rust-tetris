use serde::{Serialize, Deserialize};
use std::path::{Path};
use std::fs::{OpenOptions};
use std::io::{Write};
use std::cmp::{Ord, Ordering};
use dirs::home_dir;


#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct Score
{
    pub name: String,
    pub score: u32
}

impl Ord for Score
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

fn load_scores(path: &Path) -> Vec<Score>
{
    let file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .open(path)
    .unwrap();
    if let Ok(scores) = serde_json::from_reader(file)
    {
        return scores
    }
    else
    {
        return Vec::new();
    }

}

fn write_scores(path: &Path, scores: Vec<Score>)
{
    let ser = serde_json::to_string(&scores).unwrap();
    let mut file = OpenOptions::new()
    .write(true)
    .truncate(true)
    .create(true)
    .open(path)
    .unwrap();
    file.write_all(ser.as_bytes()).unwrap();
    file.flush().unwrap();
}

fn ask_username() -> String
{
    println!("Name:");
    //std::io::stdout().flush().unwrap();
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    buffer = buffer.trim().into();
    return buffer;
}

pub fn manage_highscore(pscore: u32)
{
    let mut path = home_dir().unwrap();
    path.push(".tetris");
    let mut scores = load_scores(path.as_path());
    if scores.iter().any(|s| pscore > s.score) || scores.is_empty()
    {
        println!("Your score: {}", pscore);
        let name = ask_username();
        scores.push(Score{name: name, score: pscore});
        scores.sort();
        scores.reverse();
        scores.truncate(10);
        print!("{}",termion::clear::BeforeCursor);
        println!("Highscores:");
        scores.iter().for_each(|score|
            println!("{0}: {1}", score.name, score.score));
        write_scores(path.as_path(), scores);
    }
    
    
}

pub fn print_highscores()
{
    let mut path = home_dir().unwrap();
    path.push(".tetris");
    let scores = load_scores(path.as_path());
    print!("{}",termion::clear::BeforeCursor);
    scores.iter().for_each(|score|
        println!("{0}: {1}", score.name, score.score));
}