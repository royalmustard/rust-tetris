use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use dirs::home_dir;


#[derive(Serialize, Deserialize)]
struct Score
{
    name: String,
    score: u32
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
    println!("{}", buffer);
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
        write_scores(path.as_path(), scores);
        print_highscores(path.as_path());
    }
    
}

pub fn print_highscores(path: &Path)
{

}