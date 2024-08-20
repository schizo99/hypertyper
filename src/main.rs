/// av alla ord som rör sig mot "skölden".. börjar du skriva på ett ord så måste du skriva klart det
/// två ord som börjar på samma bokstav får inte finnas på spelplanen samtidigt, så länge inte en spelare börjat skriva på ett ord
/// för varje bokstav som går igenom muren så förlorar du en # (sköld)
use std::{
    io,
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};
use rand::Rng;

const WIDTH: i32 = 20;
struct Word {
    word: String,
    x: i32,
    y: i32,
    started: bool,
    enabled: bool,
    completed: bool,
}

struct Player {
    name: String,
    shields: i32,
    level: i32,
    score: i32,
    screen_width: i32,
}

fn main() {
    execute!(io::stdout(), Clear(ClearType::All)).unwrap();
    execute!(io::stdout(), Hide).unwrap();
    enable_raw_mode().unwrap();
    execute!(io::stdout(), MoveTo(0, 0)).unwrap();
    println!("Welcome, press space to start the game!");
    // wait until space is pressed
    loop {
        let key = get_key();
        if key == " " {
            break;
        }
    }
    execute!(io::stdout(), Clear(ClearType::All)).unwrap();
    // get size of terminal
    let size = size().unwrap();
    let width = size.0 as i32;
    let height = size.1 as i32;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        let key = get_key();
        tx.send(key).unwrap();
    });
    let player = Player {
        name: "kalle".to_string(),
        shields: 3,
        level: 1,
        score: 0,
        screen_width: width,
    };
    fun_name(rx, height, player);
    execute!(io::stdout(), Show).unwrap();
    disable_raw_mode().unwrap();
}

fn draw_shield(width: i32, height: i32) {
    for y in 1..height {
        execute!(io::stdout(), MoveTo(width as u16, y as u16)).unwrap();
        print!("#");
    }
}

fn draw_toolbar(player: &Player) {
    execute!(io::stdout(), MoveTo(0, 0)).unwrap();
    print!("Score: {}", player.score);
    execute!(io::stdout(), MoveTo(30, 0)).unwrap();
    print!("Level: {}", player.level);
    execute!(io::stdout(), MoveTo(60, 0)).unwrap();
    print!("Shields: {}", player.shields);
}

fn randomword(width: i32, height: i32) -> Word {
    let words = ["terminal", "rust", "bajs"];
    let word = words[rand::thread_rng().gen_range(0..words.len())];
    Word {
        word: word.to_string(),
        x: width - word.len() as i32,
        y: rand::thread_rng().gen_range(1..height),
        started: false,
        enabled: false,
        completed: false,
    }
}

fn hit() {
    execute!(io::stdout(), Clear(ClearType::All)).unwrap();
    execute!(
        io::stdout(),
        SetBackgroundColor(Color::White),
        ResetColor,
    ).unwrap();
}

fn fun_name(rx: mpsc::Receiver<String>, height: i32, mut player: Player) {
    let mut winner = false;

    let sleeptime = Duration::from_millis(100 - player.level as u64);
    execute!(io::stdout(), Clear(ClearType::All)).unwrap();
    let mut words: Vec<Word> = Vec::new();
    let mut breakout = false;
    loop {
        draw_shield(WIDTH, height);
        draw_toolbar(&player);
        words.retain(|w| !w.completed);
        if words.len() < 5 + player.level as usize {
            let new_word = randomword(player.screen_width,height);
            let conflict = words
                .iter()
                .any(|w| w.word.starts_with(&new_word.word[0..1]));
            if !conflict {
                words.push(new_word);
            }
        }
        for w in words.iter_mut() {
            if !w.enabled {
                if rand::thread_rng().gen_range(0..100) < 10 {
                    w.enabled = true;
                }
            }
            if w.x < WIDTH {
                player.shields -= 1;
                if player.shields == 0 {
                    winner = false;
                    breakout = true;
                }
                w.completed = true;
            }
        }
        let mut keypressed = "kalle".to_string();
        match rx.try_recv() {
            Ok(key) => {
                if key == "\x1B" {
                    breakout = true;
                }
                keypressed = key;
            }
            Err(_) => {}
        }

        let any_started = words.iter().any(|w| w.started);

        for w in words.iter_mut() {
            if w.enabled && !w.completed {
                w.x -= 1;
                if w.word.starts_with(&keypressed) && w.started {
                    w.word = w.word[1..].to_string();
                    hit();
                } else if !w.started && !any_started {
                    if w.word.starts_with(&keypressed) {
                        w.started = true;
                        w.word = w.word[1..].to_string();
                        hit();
                    }
                }
                execute!(io::stdout(), MoveTo(WIDTH as u16, w.y as u16)).unwrap();
                execute!(io::stdout(), Clear(ClearType::CurrentLine)).unwrap();
                execute!(io::stdout(), Print(format!("#"))).unwrap();
                execute!(io::stdout(), MoveTo(w.x as u16, w.y as u16)).unwrap();
                if w.x > WIDTH {
                    execute!(io::stdout(), Print(format!("{}", &w.word))).unwrap();
                }
            }
            if w.word.len() == 0 {
                w.completed = true;
            }
        }

        sleep(sleeptime);
        if breakout {
            break;
        }
    }

    execute!(io::stdout(), Clear(ClearType::All)).unwrap();
    if winner {
        println!("You won!");
    } else {
        println!("You lost!");
    }
}

fn get_key() -> String {
    if let Event::Key(KeyEvent { code, .. }) = read().unwrap() {
        return match code {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Enter => "\n".to_string(),
            KeyCode::Backspace => "\x08".to_string(),
            KeyCode::Delete => "\x7F".to_string(),
            KeyCode::Esc => "\x1B".to_string(),
            _ => "".to_string(),
        };
    }
    "".to_string()
}
