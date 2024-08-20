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
    cursor::{Hide, MoveTo, Show}, event::{read, Event, KeyCode, KeyEvent}, execute, style::Print, terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType}
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
    fun_name(words, rx, height, width);
    execute!(io::stdout(), Show).unwrap();
    disable_raw_mode().unwrap();
}

fn draw_shield(width: u16, height: u16) {
    for y in 0..height {
                execute!(io::stdout(), MoveTo(width, y)).unwrap();
                print!("#");
            }
}

fn fun_name(
    words: [&str; 2],
    rx: mpsc::Receiver<String>,
    height: u16,
    width: u16,
) {
    let mut winner = false;
    for (i, word ) in words.iter().enumerate() {
        let sleeptime = Duration::from_millis(100-i as u64);
        execute!(io::stdout(), Clear(ClearType::All)).unwrap();
        let y = rand::thread_rng().gen_range(0..height);
        let mut x_position = 0;
        let mut modify = word.to_string();
        let mut counter = 0;
        let mut length = word.len() - counter;
        let mut letters = Vec::new();
        draw_shield(80, height);
        loop {
            execute!(io::stdout(), MoveTo(80, y)).unwrap();
            execute!(io::stdout(), Clear(ClearType::CurrentLine)).unwrap();
            for c in letters.iter() {
                modify = modify.replace(c, " ");
            }
            let a = format!("# {:>width$}", &modify, width = width as usize - 82 as usize -counter as usize);
            execute!(io::stdout(), Print(a)).unwrap();
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
            if counter > width as usize - 82 {
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
