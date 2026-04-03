mod chip8;

use std::io::Write;
use std::time::{Duration, Instant};

use chip8::Chip8;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyboardEnhancementFlags, PushKeyboardEnhancementFlags, PopKeyboardEnhancementFlags};
use crossterm::execute;

fn render(display: &[[bool; 64]; 32]) {
    print!("\x1B[H");
    for (i, row) in display.iter().enumerate() {
        for &pixel in row {
            print!("{}", if pixel { "█" } else { " " });
        }
        if i < 31 {
            print!("\r\n");
        }
    }
    let _ = std::io::stdout().flush();
}

// Ensures terminal is restored even if we panic or return early.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(std::io::stdout(), PopKeyboardEnhancementFlags);
        let _ = crossterm::terminal::disable_raw_mode();
        print!("\x1B[?25h\r\n");
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

    crossterm::terminal::enable_raw_mode()?;
    execute!(std::io::stdout(), PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES))?;
    let _guard = TerminalGuard;

    print!("\x1B[2J\x1B[?25l\x1B[H"); // clear screen, hide cursor, go to top

    let cpu_interval = Duration::from_micros(2000); // ~500Hz
    let timer_interval = Duration::from_nanos(1_000_000_000 / 60); // 60Hz
    let mut last_cpu_tick = Instant::now();
    let mut last_timer_tick = Instant::now();
    let mut last_render = Instant::now();

    loop {
        // Input handling
        if crossterm::event::poll(Duration::from_millis(0))? {
            if let Event::Key(key_event) = crossterm::event::read()? {
                if key_event.code == KeyCode::Esc {
                    break;
                }
                if key_event.kind != KeyEventKind::Repeat {
                    if let Some(chip8_key) = map_key(key_event.code) {
                        chip8.set_key(chip8_key, key_event.kind == KeyEventKind::Press);
                    }
                }
            }
        }

        // Tick Chip8 at ~500Hz
        if last_cpu_tick.elapsed() >= cpu_interval {
            let raw = chip8.fetch();
            let opcode = chip8.decode(raw);
            chip8.execute(opcode);
            last_cpu_tick = Instant::now();
        }

        // Render at ~60Hz
        if last_render.elapsed() >= timer_interval {
            render(chip8.display());
            last_render = Instant::now();
        }

        // Tick timers at 60Hz
        if last_timer_tick.elapsed() >= timer_interval {
            chip8.tick_timers();
            last_timer_tick = Instant::now();
        }
    }

    Ok(()) // TerminalGuard::drop() runs here
}

fn map_key(key: KeyCode) -> Option<usize> {
    match key {
        KeyCode::Char('1') => Some(0x1),
        KeyCode::Char('2') => Some(0x2),
        KeyCode::Char('3') => Some(0x3),
        KeyCode::Char('4') => Some(0xC),
        KeyCode::Char('q') => Some(0x4),
        KeyCode::Char('w') => Some(0x5),
        KeyCode::Char('e') => Some(0x6),
        KeyCode::Char('r') => Some(0xD),
        KeyCode::Char('a') => Some(0x7),
        KeyCode::Char('s') => Some(0x8),
        KeyCode::Char('d') => Some(0x9),
        KeyCode::Char('f') => Some(0xE),
        KeyCode::Char('z') => Some(0xA),
        KeyCode::Char('x') => Some(0x0),
        KeyCode::Char('c') => Some(0xB),
        KeyCode::Char('v') => Some(0xF),
        _ => None,
    }
}
