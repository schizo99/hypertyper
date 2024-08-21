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
        Color, Colors, Print, PrintStyledContent, ResetColor, SetBackgroundColor, SetColors, SetForegroundColor, Stylize
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
        screen_width: width,
    };
    fun_name(rx, height, &mut player);
    add_highscore(&args, &player);
    enable_raw_mode().unwrap();
    show_highscore(&args.path, &player);
    get_key.join().unwrap();
    disable_raw_mode().unwrap();
    execute!(io::stdout(), Show).unwrap();
}

fn draw_shield(width: i32, height: i32) {
    for y in 2..height {
        execute!(io::stdout(), MoveTo(width as u16, y as u16)).unwrap();
        print!("#");
    }
}

fn draw_toolbar(player: &Player) {
    execute!(io::stdout(), MoveTo(5, 1), Clear(ClearType::CurrentLine)).unwrap();
    print!("Score: {}", player.score);
    execute!(io::stdout(), MoveTo(35, 1)).unwrap();
    print!("Level: {}", player.level);
    execute!(io::stdout(), MoveTo(65, 1)).unwrap();
    print!("Shields: {}", ")".repeat(player.shields as usize));
}

fn draw_border(width: i32, height: i32) {
    for x in 0..width - 1 {
        execute!(io::stdout(), MoveTo(x as u16, 0)).unwrap();
        print!("#");
        execute!(io::stdout(), MoveTo(x as u16, 2)).unwrap();
        print!("#");
        execute!(io::stdout(), MoveTo(x as u16, height as u16 - 1)).unwrap();
        print!("#");
    }
    for y in 0..height - 1 {
        execute!(io::stdout(), MoveTo(0, y as u16)).unwrap();
        print!("#");
        execute!(io::stdout(), MoveTo(width as u16 - 1, y as u16)).unwrap();
        print!("#");
    }
}

fn randomword(width: i32, height: i32) -> Word {
    let words = ["terminal", "rust", "bajs"];
    let word = words[rand::thread_rng().gen_range(0..words.len())];
    Word {
        word: word.to_string(),
        x: width - 2,
        y: rand::thread_rng().gen_range(3..height - 1),
        started: false,
        enabled: false,
        completed: false,
        hit: false,
    }
}

fn display_word(word: &Word, player: &Player) -> String{
    let wordlength = word.word.len() + word.x as usize;
    let overflow: i32 = wordlength as i32 - player.screen_width;
    if overflow < 0 {
        return word.word.to_string();
    } else {
        return word.word[0..word.word.len() - overflow as usize].to_string();
    }
}

fn fun_name(rx: mpsc::Receiver<String>, height: i32, player: &mut Player) {
    execute!(io::stdout(), Clear(ClearType::All)).unwrap();
    let mut words: Vec<Word> = Vec::new();
    let mut breakout = false;
    loop {
        if breakout {
            break;
        }
        if player.score % 5 == 0 {
            player.level = player.score / 5;
        }
        let sleeptime = Duration::from_millis(1000 - (player.level * 2) as u64);
        draw_shield(WIDTH, height);
        draw_toolbar(&player);
        words.retain(|w| !w.completed);
        if words.len() < 5 + player.level as usize {
            let new_word = randomword(player.screen_width, height);
            let conflict = words
                .iter()
                .any(|w| w.word.starts_with(&new_word.word[0..1]));
            if !conflict {
                words.push(new_word);
            }
        }
        for w in words.iter_mut() {
            if !w.enabled {
                if rand::thread_rng().gen_range(0..100) < 50 {
                    w.enabled = true;
                }
            }
            if w.x <= WIDTH {
                execute!(io::stdout(), EnterAlternateScreen).unwrap();
                execute!(io::stdout(), SetBackgroundColor(Color::White)).unwrap();
                execute!(io::stdout(), Clear(ClearType::All)).unwrap();
                sleep(Duration::from_millis(20));
                execute!(io::stdout(), LeaveAlternateScreen).unwrap();
                player.shields -= 1;
                if player.shields == 0 {
                    breakout = true;
                }
                if w.word.len() > 0 {
                    w.x += 1;
                    w.word = w.word[1..].to_string();
                } else {
                    w.completed = true;
                }
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
                    w.hit = true;
                } else if !w.started && !any_started {
                    if w.word.starts_with(&keypressed) {
                        w.started = true;
                        w.word = w.word[1..].to_string();
                        w.hit = true;
                    }
                }
                execute!(io::stdout(), MoveTo(w.x as u16, w.y as u16)).unwrap();
                //execute!(io::stdout(), Clear(ClearType::CurrentLine)).unwrap();
                let word = display_word(&w, &player);
                let barrier_collision = format!("{}", word);

                if w.hit || w.started{
                    execute!(
                        io::stdout(),
                        SetForegroundColor(Color::DarkRed),
                        Print(barrier_collision),
                        SetColors(Colors::new(Color::Reset, Color::Reset)),
                        PrintStyledContent("  ".white()),
                        ResetColor
                    )
                    .unwrap();
                    w.hit = false;
                } else {
                    execute!(
                        io::stdout(),
                        Print(barrier_collision),
                       PrintStyledContent(" ".white()),
                    )
                    .unwrap();
                }
                execute!(io::stdout(), MoveTo(player.screen_width as u16, w.y as u16)).unwrap();
                execute!(io::stdout(), Print(format!("#"))).unwrap();
            }
            if w.word.len() == 0 {
                w.completed = true;
                player.score += 1;
            }
        }
        draw_border(player.screen_width, height);

        sleep(sleeptime);
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
