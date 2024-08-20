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
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};
use rand::Rng;

struct Word {
    word: String,
    x: u16,
    y: u16,
    started: bool,
    enabled: bool,
    completed: bool,
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
    let width = size.0;
    let height = size.1;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        let key = get_key();
        tx.send(key).unwrap();
    });
    fun_name(rx, height, width);
    execute!(io::stdout(), Show).unwrap();
    disable_raw_mode().unwrap();
}

fn draw_shield(width: u16, height: u16) {
    for y in 0..height {
        execute!(io::stdout(), MoveTo(width, y)).unwrap();
        print!("#");
    }
}

fn fun_name(rx: mpsc::Receiver<String>, height: u16, width: u16) {
    let mut winner = false;

    let sleeptime = Duration::from_millis(100);
    execute!(io::stdout(), Clear(ClearType::All)).unwrap();
    let mut x_position = 0;
    // let mut modify = word.to_string();
    let mut counter = 0;
    // let mut length = word.len() - counter;
    let mut words: Vec<Word> = Vec::new();
    for w in ["terminal", "rust"] {
        let word = Word {
            word: w.to_string(),
            x: x_position,
            y: rand::thread_rng().gen_range(0..height),
            started: false,
            enabled: false,
            completed: false,
        };
        words.push(word);
    }
    draw_shield(80, height);
    let mut breakout = false;
    let mut level = 1;
    loop {
        words.retain(|w| !w.completed);
        if words.len() == 0 {
            winner = true;
            break;
        }
        for w in words.iter_mut() {
            if !w.started {
                if rand::thread_rng().gen_range(0..100) < 10 {
                    w.enabled = true;
                }
            }
        }
        for w in words.iter_mut() {
            if w.enabled && !w.completed {
                execute!(io::stdout(), MoveTo(80, w.y)).unwrap();
                execute!(io::stdout(), Clear(ClearType::CurrentLine)).unwrap();
                let a = format!(
                    "# {:>width$}",
                    &w.word[0..],
                    width = width as usize - 82 as usize - counter as usize
                );
                execute!(io::stdout(), Print(a)).unwrap();
                counter += 1;
                match rx.try_recv() {
                    Ok(key) => {
                        if key == "\x1B" {
                            breakout = true;
                        }
                        if w.word.starts_with(&key) {
                            w.started = true;
                            w.word = w.word[1..].to_string();
                            if w.word.len() == 0 {
                                w.completed = true;
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
        }
        sleep(sleeptime);
        if counter > width as usize - 82 {
            winner = false;
            break;
        }
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
