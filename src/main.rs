use std::{
    io,
    ops::Deref,
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};
use rand::Rng;

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
    let words = ["terminal", "rust"];
    
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        let key = get_key();
        tx.send(key).unwrap();
    });

    fun_name(words, rx, height);

    disable_raw_mode().unwrap();
}

fn fun_name(
    words: [&str; 2],
    rx: mpsc::Receiver<String>,
    height: u16,
) {
    let mut winner = false;
    for (i, word ) in words.iter().enumerate() {
        let sleeptime = Duration::from_millis(100-i as u64);
        execute!(io::stdout(), Clear(ClearType::All)).unwrap();
        let y = rand::thread_rng().gen_range(0..height);
        let mut x_position = 0;
        let mut modify = word.to_string();
        let mut counter = 1;
        let mut length = word.len() - counter;
        let mut letters = Vec::new();
        loop {
            execute!(io::stdout(), MoveTo(x_position, y)).unwrap();
            execute!(io::stdout(), Clear(ClearType::CurrentLine)).unwrap();
            if length <= 0 {
                length = 0;
                x_position += 1;
            } else {
                length = word.len() - counter;
            }
            for c in letters.iter() {
                modify = modify.replace(c, " ");
            }
            println!("{}", &modify[length..]);
            sleep(sleeptime);
            counter += 1;
            match rx.try_recv() {
                Ok(key) => {
                    if key == "\x1B" {
                        break;
                    }
                    if word.contains(&key) {
                        letters.push(key);
                        if word.chars().all(|c| letters.contains(&c.to_string())) {
                            winner = true;
                            break;
                        }
                    }
                }
                Err(_) => {
                }
            }
            if counter > 50 {
                break;
            }
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
