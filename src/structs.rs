use argh::FromArgs;
use rand::Rng;

pub const SHIELD_POSITION: i32 = 15;
pub const INITIAL_SPEED: i32 = 15000;
pub const MAX_SHIELDS: i32 = 15;

pub const WORDS: &str = include_str!("words_alpha.txt");
pub const SPLASH: &str = include_str!("splash.txt");

/// Command line arguments for the application.
#[derive(FromArgs, Debug)]
#[argh(description = "Command line arguments for the application.")]
pub struct Args {
    #[argh(option, short = 'u', default = "String::from(\"show_highscore\")")]
    /// username
    pub username: String,

    #[argh(option, short = 'p', default = "String::from(\"highscore.txt\")")]
    /// path to highscore file
    pub path: String,

    #[argh(short = 's', switch)]
    /// show highscore
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

impl Word {
    pub fn new(word: &String, field: &Field) -> Self {
        Self {
            word: word.to_string(),
            original_word: word.to_string(),
            x: field.width - 2,
            y: rand::thread_rng().gen_range(5..field.height - 3),
            started: false,
            enabled: true,
            completed: false,
            hit: false,
        }
    }
}

pub struct Player {
    pub name: String,
    pub shields: i32,
    pub level: i32,
    pub score: i32,
    pub is_alive: bool,
}

impl Player {
    pub fn new(name: String) -> Self {
        Self {
            name,
            shields: MAX_SHIELDS,
            level: 1,
            score: 0,
            is_alive: true,
        }
    }
}
