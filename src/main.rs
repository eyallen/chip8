mod chip8;

use std::time::Duration;

use chip8::Chip8;
use crossterm::event::{Event, KeyCode, KeyEventKind, poll};

fn render(display: &[[bool; 64]; 32]) {
    print!("\x1B[H");
    for row in display {
        for &pixel in row {
            print!("{}", if pixel { "█" } else { " " });
        }
        println!();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut chip8 = Chip8::new();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: chip8 <rom>");
        std::process::exit(1);
    }

    chip8.load_rom(&args[1]).expect("Failed to load ROM");

    print!("\x1B[2J"); // clear screen once at start
    loop {
        // Input handling
        if crossterm::event::poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = crossterm::event::read()? {
                let pressed = key_event.kind == KeyEventKind::Press;
                if let Some(chip8_key) = map_key(key_event.code) {
                    chip8.set_key(chip8_key, pressed);
                }
            }
        }

        // Tick Chip8
        let raw = chip8.fetch();
        let opcode = chip8.decode(raw);
        chip8.execute(opcode);
        render(chip8.display());

        // Slow our render loop down
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn map_key(key: KeyCode) -> Option<usize> {
    match key {
        KeyCode::Char('1') => Some (0x1),
        KeyCode::Char('2') => Some (0x2),
        KeyCode::Char('3') => Some (0x3),
        KeyCode::Char('4') => Some (0xC),
        KeyCode::Char('Q') => Some (0x4),
        KeyCode::Char('W') => Some (0x5),
        KeyCode::Char('E') => Some (0x6),
        KeyCode::Char('R') => Some (0xD),
        KeyCode::Char('A') => Some (0x7),
        KeyCode::Char('S') => Some (0x8),
        KeyCode::Char('D') => Some (0x9),
        KeyCode::Char('F') => Some (0xE),
        KeyCode::Char('Z') => Some (0xA),
        KeyCode::Char('X') => Some (0x0),
        KeyCode::Char('C') => Some (0xB),
        KeyCode::Char('V') => Some (0xF),
        _ => None
    }
}