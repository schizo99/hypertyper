/// Module for handling highscores in the game.
///
/// This module provides functions for adding highscores, validating highscore files,
/// showing highscores, and handling highscore commands.
///
use crate::structs::{Args, Player};
use clap::CommandFactory;
use crossterm::{
    cursor::{Hide, MoveTo},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
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

fn validate_highscore_file(path: &str) {
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

pub fn show_highscore(path: &str, player: &Player) {
    let end_score = player.score;

    let content = top_highscores(&path).join("\n");
    disable_raw_mode().expect("Failed to disable raw mode");
    execute!(io::stdout(), Clear(ClearType::All)).expect("Failed to clear screen");
    execute!(io::stdout(), MoveTo(0, 0)).expect("Failed to move cursor");
    execute!(io::stdout(), Hide).expect("Failed to hide cursor");
    if player.name == "show_highscore" {
        println!("{}\n (Press any key to continue...)", &content);
    } else {
        println!(
            "{}\n You scored {} points and made it to level {} ",
            &content, end_score, player.level
        );
        println!(" (Press any key to continue...)");
    }
    enable_raw_mode().expect("Failed to enable raw mode");
}

pub fn handle_highscore(args: &Args) {
    let username = &args.username;
    if username == "show_highscore" && !args.show_highscore {
        Args::command().print_help().unwrap();
        std::process::exit(0);
    }
    let path = &args.path;
    validate_highscore_file(&path);
    if args.show_highscore {
        show_highscore(
            &path,
            &Player {
                name: "show_highscore".to_string(),
                score: 0,
                shields: 3,
                max_shields: 15,
                level: 0,
                screen_width: 0,
                screen_height: 0,
                is_alive: true,
            },
        );
        std::process::exit(0);
    }
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
