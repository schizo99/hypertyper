use clap::Parser;

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

pub const WIDTH: i32 = 60;
pub struct Word {
    pub word: String,
    pub x: i32,
    pub y: i32,
    pub started: bool,
    pub enabled: bool,
    pub completed: bool,
    pub hit: bool,
}

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub shields: i32,
    pub level: i32,
    pub score: i32,
    pub screen_width: i32,
}
