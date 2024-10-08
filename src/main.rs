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
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{
        Color, Colors, Print, PrintStyledContent, ResetColor, SetBackgroundColor, SetColors,
        SetForegroundColor, Stylize,
    },
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use highscore::*;
use rand::Rng;
use structs::*;
fn main() {
    let args = Args::parse();
    validera_highscore_file(&args.path);
    if args.show_highscore {
        highscore(&args);
    }
    intro();
    let (rx, get_key) = key_thread();
    let mut player = Player::new(args.username.to_string());
    let dictionary = WORDS.lines().map(|s| s.to_string()).collect();
    mamma(rx, &mut player, &dictionary);
    add_highscore(&args, &player);
    show_highscore(&args.path);
    println!(
        " (You scored {} points and made it to level {}) ",
        player.score, player.level
    );
    println!(" (Press any key to continue...)");
    enable_raw_mode().unwrap();
    get_key.join().unwrap();
    disable_raw_mode().unwrap();
    execute!(io::stdout(), Show).unwrap();
}

fn highscore(args: &Args) {
    show_highscore(&args.path);
    println!("(Press any key to continue...)");
    enable_raw_mode().expect("Failed to enable raw mode");
    read().unwrap();
    disable_raw_mode().expect("Failed to disable raw mode");
    execute!(io::stdout(), Show).expect("Failed to show cursor");
    std::process::exit(0);
}

fn key_thread() -> (mpsc::Receiver<String>, thread::JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let get_key = thread::spawn(move || loop {
        let key = get_key();
        match tx.send(key) {
            Ok(_) => {}
            Err(_) => {
                return;
            }
        }
        sleep(Duration::from_millis(10));
    });
    (rx, get_key)
}

fn intro() {
    execute!(io::stdout(), Clear(ClearType::All)).unwrap();
    execute!(io::stdout(), MoveTo(0, 0)).unwrap();

    // Set foreground color to red
    execute!(io::stdout(), SetForegroundColor(Color::Red)).unwrap();

    // Read the splash.txt file and display it on screen
    println!("{}", SPLASH.to_string());

    // Reset the color to default
    execute!(io::stdout(), ResetColor).unwrap();
    execute!(io::stdout(), Hide).unwrap();
    enable_raw_mode().unwrap();

    // Sleep for 500 ms
    sleep(Duration::from_millis(500));

    println!(
        "{}(Welcome! Press space to start the game!)",
        " ".repeat(45)
    );
    // wait until a key is pressed
    read().unwrap();
    execute!(io::stdout(), Clear(ClearType::All)).unwrap();
}

fn draw_shield(field: &Field) {
    execute!(io::stdout(), SetForegroundColor(Color::Red)).unwrap();
    execute!(io::stdout(), MoveTo(SHIELD_POSITION as u16, 3)).unwrap();
    println!("|");
    execute!(io::stdout(), MoveTo(SHIELD_POSITION as u16, 4)).unwrap();
    println!("v");
    execute!(io::stdout(), ResetColor).unwrap();

    execute!(io::stdout(), SetForegroundColor(Color::Blue)).unwrap();
    for y in 5..field.height - 3 {
        execute!(io::stdout(), MoveTo(SHIELD_POSITION as u16, y as u16)).unwrap();
        print!("#");
    }
    execute!(io::stdout(), ResetColor).unwrap();
    execute!(
        io::stdout(),
        MoveTo(SHIELD_POSITION as u16, field.height as u16 - 3)
    )
    .unwrap();
    execute!(io::stdout(), SetForegroundColor(Color::Red)).unwrap();
    println!("^");
    execute!(
        io::stdout(),
        MoveTo(SHIELD_POSITION as u16, field.height as u16 - 2)
    )
    .unwrap();
    println!("|");
    execute!(io::stdout(), ResetColor).unwrap();
}

fn draw_toolbar(player: &Player) {
    execute!(io::stdout(), MoveTo(5, 1), Clear(ClearType::CurrentLine)).unwrap();

    println!("Score: {}", player.score);
    execute!(io::stdout(), MoveTo(25, 1)).unwrap();
    println!("Level: {}", player.level);
    execute!(io::stdout(), MoveTo(47, 1)).unwrap();
    print!("Shields: [ ");
    for _ in 0..MAX_SHIELDS {
        print!(".");
    }
    println!(" ]");

    execute!(io::stdout(), MoveTo(58, 1)).unwrap();
    for _ in 0..player.shields {
        print!(">");
    }
}

fn draw_border(field: &Field) {
    // Set color to dark grey
    execute!(io::stdout(), SetForegroundColor(Color::DarkGrey)).unwrap();

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
    execute!(io::stdout(), MoveTo(0, field.height as u16 - 1)).unwrap();

    print!("\\");
    print!("{}", "-".repeat(field.width as usize - 2));
    print!("/");
    // Reset the color to default
    execute!(io::stdout(), ResetColor).unwrap();
}

fn randomword(field: &Field, wordlist: &Vec<String>, player: &Player) -> Word {
    loop {
        let word = &wordlist[rand::thread_rng().gen_range(0..wordlist.len() as usize)];
        if word.len() <= player.level as usize + 4 {
            return Word::new(word, field);
        }
    }
}

fn update_words(key: String, words: &mut Vec<Word>, player: &mut Player) -> bool {
    // Check if key is empty or space or does not contai a character
    if key.trim().is_empty() {
        return false;
    }

    let any_word_started = words.iter().any(|w| w.started);

    for word in words.iter_mut() {
        if word.word.starts_with(&key) && word.started {
            word.word = word.word[1..].to_string();
            word.x += 1;
            player.score += 1;
            return true;
        } else if word.word.starts_with(&key) && !any_word_started {
            word.started = true;
            word.word = word.word[1..].to_string();
            word.x += 1;
            player.score += 1;
            return true;
        }
    }
    false
}
fn mamma(rx: mpsc::Receiver<String>, player: &mut Player, dictionary: &Vec<String>) {
    let field = Field {
        width: 80,
        height: 24,
    };
    let mut words: Vec<Word> = Vec::new();
    let mut gametick = 1;
    while player.is_alive {
        add_word(&field, &mut words, &dictionary, player);

        match rx.try_recv() {
            Ok(key) => {
                if key == "EXIT" {
                    player.is_alive = false;
                }
                _ = {
                    if key.to_string().is_ascii() && key.len() == 1 && !key.trim().is_empty() {
                        let updated = update_words(key.clone(), &mut words, player);
                        if updated {
                            draw_words(&mut words, &field);
                            draw_border(&field);
                        }
                    }
                };
            }
            Err(_) => {}
        }
        if player.score > 0 && player.score / 75 > 0 {
            player.level = player.score / 75 + 1;
        }
        if gametick % 6 == 0 {
            word_completed(&mut words, player);
            draw_words(&mut words, &field);
            draw_border(&field);
            draw_toolbar(player);
            move_words(&mut words);
            draw_shield(&field);
            shield_hit(&mut words, player);
            words.retain(|w| !w.completed);
        }
        gametick += 1;
        let speed = calculate_speed(player);
        let sleep_time = Duration::from_micros(speed as u64);
        sleep(Duration::from_micros(sleep_time.as_micros() as u64));
    }
    drop(rx);
    end_game();
}

fn word_completed(words: &mut Vec<Word>, player: &mut Player) {
    for word in &mut *words {
        if word.completed {
            if player.shields < MAX_SHIELDS {
                player.shields += 1;
            }
        }
    }
    words.retain(|w| !w.completed);
}

fn calculate_speed(player: &mut Player) -> i32 {
    INITIAL_SPEED - player.level * 300 as i32
}

fn add_word(field: &Field, words: &mut Vec<Word>, dictionary: &Vec<String>, player: &Player) {
    let mut new_word = randomword(field, dictionary, player);
    if words.len() < 1 {
        words.push(new_word);
    } else if words.len() < player.level as usize + 1 + (player.level as usize / 6) {
        let mut conflict = words
            .iter()
            .any(|w| w.original_word.starts_with(&new_word.word[0..1]));
        while conflict {
            new_word = randomword(field, dictionary, player);
            let distance = words
                .iter()
                .map(|w| w.x as i32 - w.word.len() as i32)
                .max()
                .unwrap()
                < new_word.x - rand::thread_rng().gen_range(10..20);
            let collision = words.iter().any(|w| w.y == new_word.y);
            conflict = words
                .iter()
                .any(|w| w.word.starts_with(&new_word.word[0..1]));
            if !conflict && !collision && distance {
                words.push(new_word);
            }
        }
    }
}

fn shield_hit(words: &mut Vec<Word>, player: &mut Player) {
    if player.shields <= 0 {
        player.is_alive = false;
    }
    for word in words.iter_mut() {
        if word.x <= SHIELD_POSITION - 1 && word.word.len() > 0 {
            word.hit = true;
            player.shields -= 1;
            word.word = word.word[1..].to_string();
            word.x += 1;
        }
    }
}

fn move_words(words: &mut Vec<Word>) {
    for word in words.iter_mut() {
        word.x -= 1;
    }
}

fn draw_words(words: &mut Vec<Word>, field: &Field) {
    for word in words {
        let word2 = format!(" {}", truncate_word(word, field.width - 1));
        draw_word(word, word2, field.width);
    }
}

fn draw_word(word: &mut Word, truncated_word: String, width: i32) {
    if word.enabled && !word.completed {
        execute!(io::stdout(), MoveTo(word.x as u16, word.y as u16)).unwrap();
        if word.hit {
            execute!(
                io::stdout(),
                SetForegroundColor(Color::DarkRed),
                Print(truncated_word),
                SetColors(Colors::new(Color::Reset, Color::Reset)),
                PrintStyledContent("  ".white()),
                ResetColor
            )
            .unwrap();
            execute!(io::stdout(), MoveTo(width as u16 - 1, word.y as u16)).unwrap();
        } else if word.started & !word.hit {
            execute!(
                io::stdout(),
                SetForegroundColor(Color::Yellow),
                Print(truncated_word),
                SetColors(Colors::new(Color::Reset, Color::Reset)),
                PrintStyledContent("  ".white()),
                ResetColor
            )
            .unwrap();
            execute!(io::stdout(), MoveTo(width as u16 - 1, word.y as u16)).unwrap();
        } else {
            execute!(
                io::stdout(),
                Print(truncated_word),
                PrintStyledContent("  ".white()),
            )
            .unwrap();
        }
        if word.word.len() == 0 {
            word.completed = true;
        }
    }
}

fn truncate_word(word: &mut Word, width: i32) -> String {
    let wordlength = word.word.len() as i32 + word.x;
    let overflow = wordlength - width;
    if overflow < 0 {
        return word.word.to_string();
    } else {
        return word.word[0..word.word.len() - overflow as usize].to_string();
    }
}

fn get_key() -> String {
    if let Event::Key(KeyEvent {
        code, modifiers, ..
    }) = read().unwrap()
    {
        if modifiers.is_empty() {
            return match code {
                KeyCode::Char(c) => c.to_string(),
                KeyCode::Esc => "EXIT".to_string(),
                _ => "".to_string(),
            };
        } else {
            if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                disable_raw_mode().unwrap();
                execute!(io::stdout(), Show).unwrap();
                std::process::exit(0);
            }
        }
    }
    "".to_string()
}

fn end_game() {
    disable_raw_mode().unwrap();

    println!("");
    execute!(io::stdout(), MoveTo(40, 12)).unwrap();
    execute!(io::stdout(), SetBackgroundColor(Color::White)).unwrap();
    execute!(io::stdout(), SetForegroundColor(Color::Black)).unwrap();
    println!("Game over!");
    execute!(io::stdout(), ResetColor).unwrap();

    sleep(Duration::from_millis(400));
    execute!(io::stdout(), MoveTo(26, 13)).unwrap();
    println!(" (You were just too slow! Bummer...) ");
    sleep(Duration::from_millis(2000));
}
