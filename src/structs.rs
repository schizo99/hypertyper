use clap::Parser;

pub const SHIELD_POSITION: i32 = 15;
pub const INITIAL_SPEED: i32 = 100000;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Username
    #[arg(short, long, default_value = "show_highscore")]
    pub username: String,

    /// Path to highscore file
    #[arg(short, long, default_value = "highscore.txt")]
    pub path: String,

    /// Show highscore
    #[arg(short, long)]
    pub show_highscore: bool,
}

pub struct Field {
    pub width: i32,
    pub height: i32,
}
#[derive(Clone)]
pub struct Word {
    pub word: String,
    pub original_word: String,
    pub x: i32,
    pub y: i32,
    pub started: bool,
    pub enabled: bool,
    pub completed: bool,
    pub hit: bool,
}

pub struct Player {
    pub name: String,
    pub shields: i32,
    pub level: i32,
    pub score: i32,
    pub is_alive: bool,
}
