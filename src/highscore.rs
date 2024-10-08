/// Module for handling highscores in the game.
///
/// This module provides functions for adding highscores, validating highscore files,
/// showing highscores, and handling highscore commands.
///
use crate::structs::{Args, Player};

use crossterm::{
    cursor::{Hide, MoveTo},
    execute,
    terminal::{Clear, ClearType},
};
use std::fs::OpenOptions;
use std::io::{self, prelude::*};

pub fn add_highscore(args: &Args, player: &Player) {
    let mut file = OpenOptions::new().append(true).open(&args.path).unwrap();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if let Err(e) = writeln!(
        file,
        "{};{};{};{}",
        player.name, player.score, player.level, timestamp
    ) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

pub fn validera_highscore_file(path: &str) {
    println!("Validating highscore file at path: {}", path);
    match std::fs::read_to_string(path) {
        Ok(_) => {
            return;
        }
        Err(_) => match std::fs::write(path, "") {
            Ok(_) => {
                println!("Highscorefile created successfully");
            }
            Err(err) => {
                eprintln!("Error creating highscore file {}: {}", path, err);
            }
        },
    }
}

pub fn show_highscore(path: &str) {
    let content = top_highscores(&path).join("\n");
    execute!(io::stdout(), Clear(ClearType::All)).expect("Failed to clear screen");
    execute!(io::stdout(), MoveTo(0, 0)).expect("Failed to move cursor");
    execute!(io::stdout(), Hide).expect("Failed to hide cursor");
    println!("{}", &content);
}

fn top_highscores(path: &str) -> Vec<String> {
    let mut highscores: Vec<(String, i32, i32)> = Vec::new();
    let content = std::fs::read_to_string(path).unwrap();
    for line in content.lines() {
        let parts: Vec<&str> = line.split(";").collect();
        if parts.len() == 4 {
            let username = parts[0];
            let score = parts[1].parse::<i32>().unwrap();
            let level = parts[2].parse::<i32>().unwrap();
            highscores.push((username.to_string(), score, level));
        }
    }
    let mut padding = highscores
        .iter()
        .map(|(username, _, _)| username.len())
        .max()
        .unwrap_or(0);
    if padding < 6 {
        padding = 6;
    }
    highscores.sort_by(|a, b| b.1.cmp(&a.1));
    let mut result = vec![];
    result.push(format!(" Top 10 highscores:"));
    result.push(format!(" {}", "-".repeat(padding + 30)));
    result.push(format!(
        " Player{}\tScore\t\tLevel",
        " ".repeat(padding - 6)
    ));
    result.push(format!(" {}", "-".repeat(padding + 30)));
    for (_, (username, score, level)) in highscores.iter().take(10).enumerate() {
        result.push(format!(
            " {}{}\t {}\t\t {}",
            username,
            " ".repeat(padding - username.len()),
            score,
            level
        ));
    }
    result.push(format!(" {}", "-".repeat(padding + 30)));
    return result;
}
