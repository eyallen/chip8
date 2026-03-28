mod chip8;

use chip8::Chip8;

fn render(display: &[[bool; 64]; 32]) {
    print!("\x1B[H");
    for row in display {
        for &pixel in row {
            print!("{}", if pixel { "█" } else { " " });
        }
        println!();
    }
}

fn main() {
    let mut chip8 = Chip8::new();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: chip8 <rom>");
        std::process::exit(1);
    }

    chip8.load_rom(&args[1]).expect("Failed to load ROM");

    print!("\x1B[2J"); // clear screen once at start
    loop {
        let raw = chip8.fetch();
        let opcode = chip8.decode(raw);
        chip8.execute(opcode);
        render(chip8.display());
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
