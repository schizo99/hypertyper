/// av alla ord som rör sig mot "skölden".. börjar du skriva på ett ord så måste du skriva klart det
/// två ord som börjar på samma bokstav får inte finnas på spelplanen samtidigt, så länge inte en spelare börjat skriva på ett ord
/// för varje bokstav som går igenom muren så förlorar du en # (sköld)
use std::{
    io::{self},
    sync::mpsc,
    thread::{self, sleep},
    time::Duration,
};
mod highscore;
mod structs;
use clap::Parser;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    style::{
        Color, Colors, Print, PrintStyledContent, ResetColor, SetBackgroundColor, SetColors,
        SetForegroundColor, Stylize,
    },
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use highscore::*;
use rand::Rng;
use structs::*;

fn main() {
    let args = Args::parse();
    handle_highscore(&args);
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
    let get_key = thread::spawn(move || loop {
        let key = get_key();
        match tx.send(key) {
            Ok(_) => {}
            Err(_) => {
                return;
            }
        }
    });
    let mut player = Player {
        name: args.username.to_string(),
        shields: 10,
        level: 1,
        score: 0,
        screen_width: 100,
        screen_height: 30,
        is_alive: true,
    };
    let dictionary = get_dictionary_from_file();
    mamma(rx, &mut player, &dictionary);
    add_highscore(&args, &player);
    enable_raw_mode().unwrap();
    show_highscore(&args.path, &player);
    get_key.join().unwrap();
    disable_raw_mode().unwrap();
    execute!(io::stdout(), Show).unwrap();
}

fn draw_shield(field: &Field) {
    for y in 3..field.height - 1 {
        execute!(io::stdout(), MoveTo(SHIELD_POSITION as u16, y as u16)).unwrap();
        print!("#");
    }
}

fn draw_toolbar(player: &Player) {
    execute!(io::stdout(), MoveTo(5, 1), Clear(ClearType::CurrentLine)).unwrap();
    println!("Score: {}", player.score);
    execute!(io::stdout(), MoveTo(25, 1)).unwrap();
    println!("Level: {}", player.level);
    execute!(io::stdout(), MoveTo(50, 1)).unwrap();
    if player.shields >= 0 {
        println!("Shields: {}", ")".repeat(player.shields as usize));
    }
}

fn draw_border(field: &Field) {
    execute!(io::stdout(), MoveTo(0, 2)).unwrap();
    print!("/");
    print!("{}", "-".repeat(field.width as usize - 2));
    print!("\\");
    for y in 3..field.height - 1 {
        execute!(io::stdout(), MoveTo(0, y as u16)).unwrap();
        print!("|");
        execute!(io::stdout(), MoveTo(field.width as u16 - 1, y as u16)).unwrap();
        print!("|");
    }
    execute!(io::stdout(), MoveTo(0, field.height as u16 - 1 )).unwrap();

    print!("\\");
    print!("{}", "-".repeat(field.width as usize - 2));
    print!("/");

}

fn randomword(field: &Field, wordlist: &Vec<String>) -> Word {
    let word = &wordlist[rand::thread_rng().gen_range(0..wordlist.len() as usize)];
    Word {
        word: word.to_string(),
        x: field.width - 2,
        y: rand::thread_rng().gen_range(3..field.height - 1),
        started: false,
        enabled: true,
        completed: false,
        hit: false,
    }
}

fn get_dictionary_from_file() -> Vec<String> {
    let words = std::fs::read_to_string("words_alpha.txt").unwrap();
    words
        .split("\r\n")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn update_words(key: String, words: &mut Vec<Word>, player: &mut Player) {
    words.retain(|w| !w.completed || w.word.len() > 0);
    let any_word_started = words.iter().any(|w| w.started);
    for word in words.iter_mut() {
        if word.word.starts_with(&key) && word.started {
            word.word = word.word[1..].to_string();
            player.score += 1;
            word.hit = true;
        } else if !word.started && !any_word_started {
            if word.word.starts_with(&key) {
                word.started = true;
                word.word = word.word[1..].to_string();
                player.score += 1;
                word.hit = true;
            }
        }
    }
}
fn flash_screen() {
    execute!(io::stdout(), EnterAlternateScreen).unwrap();
    execute!(io::stdout(), SetBackgroundColor(Color::White)).unwrap();
    execute!(io::stdout(), Clear(ClearType::All)).unwrap();
    sleep(Duration::from_millis(10));
    execute!(io::stdout(), LeaveAlternateScreen).unwrap();
}
fn mamma(rx: mpsc::Receiver<String>, player: &mut Player, dictionary: &Vec<String>) {
    let field = Field {
        width: 80,
        height: 24,
    };
    let mut words: Vec<Word> = Vec::new();
    // read keys from user
    let mut gametick = 1;
    while player.is_alive {
        let new_word = randomword(&field, dictionary);
        if words.len() < 1 {
            words.push(new_word);
        } else if words.len() < 4 + player.level as usize {
            let conflict = words
                .iter()
                .any(|w| w.word.starts_with(&new_word.word[0..1]));
            let collision = words.iter().any(|w| w.y == new_word.y);
            if !conflict && !collision && rand::thread_rng().gen_range(0..200000) < 2 {
                words.push(new_word);
            }
        }

        match rx.try_recv() {
            Ok(key) => {
                if key == "\x1B" {
                    return;
                }
                _ = {
                    update_words(key, &mut words, player);
                };
            }
            Err(_) => {}
        }
        // update gametick
        if gametick % 5000 / player.level == 0 {
            draw_words(&mut words, &field);
            draw_border(&field);
            draw_toolbar(player);
            draw_shield(&field);
            shield_hit(&mut words, player);
        }
        gametick += 1;
        sleep(Duration::from_micros(10));
    }
    // update board every gametick
}
fn shield_hit(words: &mut Vec<Word>, player: &mut Player) {
    if player.shields == 0 {
        player.is_alive = false;
    }
    for word in words.iter_mut() {
        if word.x <= SHIELD_POSITION && word.word.len() > 0 {
            flash_screen();
            player.shields -= 1;
            word.word = word.word[1..].to_string();
            word.x += 1;
        }
    }
}
fn draw_words(words: &mut Vec<Word>, field: &Field) {
    for word in words {
        let word2 = truncate_word(word, field.width);
        draw_word(word, word2, field.width);
    }
}

fn draw_word(word: &mut Word, truncated_word: String, width: i32) {
    if word.enabled && !word.completed {
        execute!(io::stdout(), MoveTo(word.x as u16, word.y as u16)).unwrap();
        if word.x > 1 {
            word.x -= 1;
        }
        if word.hit || word.started {
            execute!(
                io::stdout(),
                SetForegroundColor(Color::DarkRed),
                Print(truncated_word),
                SetColors(Colors::new(Color::Reset, Color::Reset)),
                PrintStyledContent("  ".white()),
                ResetColor
            )
            .unwrap();
        } else {
            execute!(
                io::stdout(),
                Print(truncated_word),
                PrintStyledContent(" ".white()),
            )
            .unwrap();
        }
        if word.word.len() == 0 {
            word.completed = true;
        }
        execute!(io::stdout(), MoveTo(width as u16 - 1, word.y as u16)).unwrap();
        execute!(io::stdout(), Print(format!("#"))).unwrap();
    }
}

fn truncate_word(word: &mut Word, width: i32) -> String {
    //return word.word.to_string();
    let wordlength = word.word.len() as i32 + word.x;
    let overflow = wordlength - width;
    if overflow < 0 {
        return word.word.to_string();
    } else {
        return word.word[0..word.word.len() - overflow as usize].to_string();
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
