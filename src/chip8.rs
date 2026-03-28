use std::fs::metadata;

pub struct Chip8 {
    registers: [u8;16],
    memory: [u8;4096],
    index: u16,
    pc: u16,
    stack: [u16;16],
    sp: u8,
    delay_timer : u8,
    sound_timer: u8,
    display: [[bool;64]; 32],
}

#[derive(Debug, PartialEq)]
pub enum Opcode {
    ClearScreen,
    Jump(u16),
    SetRegister(u8,u8),
    Add(u8,u8),
    SetIndexRegister(u16),
    Draw(u8,u8,u8),
    // TODO: All the rest
}

impl Chip8 {
    const ROM_START: usize = 0x200;
    const MAX_ROM_SIZE: usize = 3584;

    const FONT_START: usize = 0x050;
    const FONT: [u8;80] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
    ];

    pub fn new() -> Self {
        let mut chip8 = Chip8 { registers: [0;16], memory: [0;4096], index: 0, pc: Self::ROM_START as u16, stack: [0;16], sp: 0, delay_timer: 0, sound_timer: 0, display: [[false;64]; 32] };


        // Load the default font at FONT_START
        chip8.memory[Self::FONT_START..Self::FONT_START + Self::FONT.len()].copy_from_slice(&Self::FONT);

        chip8
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), String> {
        let rom = std::fs::read(path).map_err(|e| e.to_string())?;

        if rom.len() > Self::MAX_ROM_SIZE {
            return Err("ROM too large".to_string());
        }

        self.memory[Self::ROM_START..Self::ROM_START + rom.len()].copy_from_slice(&rom);

        Ok(())
    }

    pub fn fetch(&mut self) -> u16 {
        let high = self.memory[self.pc as usize];
        let low = self.memory[self.pc as usize + 1];
        let opcode = (high as u16) << 8 | low as u16;

        self.pc = self.pc + 2;

        opcode
    }

    pub fn decode(&mut self, raw: u16) -> Opcode {
        let first_nibble = (raw & 0xF000) >> 12;

        // TODO: These are wrong - we need the full opcode
        // This is just to get the initial test ROM working
        match first_nibble {
            0x0 => Opcode::ClearScreen,
            0x1 => Opcode::Jump(raw & 0x0FFF),
            0x6 => Opcode::SetRegister(((raw & 0x0F00) >> 8) as u8, (raw & 0x00FF) as u8),
            0x7 => Opcode::Add(((raw & 0x0F00) >> 8) as u8, (raw & 0x00FF) as u8),
            0xA => Opcode::SetIndexRegister(raw & 0x0FFF),
            0xD => Opcode::Draw(((raw & 0x0F00) >> 8) as u8, ((raw & 0x00F0) >> 4) as u8, (raw & 0x000F) as u8),
            _ => Opcode::ClearScreen // TODO
        }
    }

    pub fn display(&self) -> &[[bool; 64]; 32] {
        &self.display
    }

    pub fn execute(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::ClearScreen => self.clear_screen(),
            Opcode::Jump(addr) => self.jump(addr),
            Opcode::SetRegister(x, nn) => self.set_register(x, nn),
            Opcode::Add(x, nn) => self.add(x, nn),
            Opcode::SetIndexRegister(addr) => self.set_index_register(addr),
            Opcode::Draw(x, y, n) => self.draw(x, y, n),
        }
    }

    fn clear_screen(&mut self) {
        self.display = [[false;64]; 32];
    }

    fn jump(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn set_register(&mut self, x: u8, nn: u8) {
        self.registers[x as usize] = nn;
    }

    fn add(&mut self, x: u8, nn: u8) {
        self.registers[x as usize] += nn;
    }

    fn set_index_register(&mut self, addr: u16) {
        self.index = addr;
    }

    fn draw(&mut self, x: u8, y: u8, n: u8) {
        let vx = self.registers[x as usize] % 64;
        let vy = self.registers[y as usize] % 32;

        self.registers[0xF] = 0;

        for row in (0..n) {
            let sprite_byte = self.memory[(self.index + row as u16) as usize];
            for col in (0..8) {
                let sprite_bit = (sprite_byte >> (7-col)) & 1;
                
                if sprite_bit == 1 {
                    let px = (vx as usize + col) % 64;
                    let py = (vy as usize + row as usize) % 32;
                    // If the sprite bit is on and the pixel at x,y is on we have a collision
                    if self.display[py][px] {
                        self.registers[0xF] = 1;
                    }
                    self.display[py][px] ^= true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_new_pc_at_rom_start() {
        let chip8 = Chip8::new();
        assert_eq!(chip8.pc, Chip8::ROM_START as u16);
    }

    #[test]
    fn test_new_font_loaded() {
        let chip8 = Chip8::new();
        assert_eq!(&chip8.memory[Chip8::FONT_START..Chip8::FONT_START + 80], &Chip8::FONT);
    }

    #[test]
    fn test_load_rom_too_large() {
        let mut chip8 = Chip8::new();
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&vec![0u8; Chip8::MAX_ROM_SIZE + 1]).unwrap();
        let result = chip8.load_rom(file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_rom_into_memory() {
        let mut chip8 = Chip8::new();
        let mut file = NamedTempFile::new().unwrap();
        let rom_data = vec![0x12, 0x34, 0x56];
        file.write_all(&rom_data).unwrap();
        chip8.load_rom(file.path().to_str().unwrap()).unwrap();
        assert_eq!(&chip8.memory[Chip8::ROM_START..Chip8::ROM_START + 3], &rom_data[..]);
    }

    #[test]
    fn test_fetch_returns_opcode() {
        let mut chip8 = Chip8::new();
        chip8.memory[Chip8::ROM_START] = 0xD1;
        chip8.memory[Chip8::ROM_START + 1] = 0x23;
        let opcode = chip8.fetch();
        assert_eq!(opcode, 0xD123);
    }

    #[test]
    fn test_fetch_increments_pc() {
        let mut chip8 = Chip8::new();
        chip8.fetch();
        assert_eq!(chip8.pc, Chip8::ROM_START as u16 + 2);
    }

    #[test]
    fn test_fetch_advances_on_successive_calls() {
        let mut chip8 = Chip8::new();
        chip8.memory[Chip8::ROM_START] = 0xAB;
        chip8.memory[Chip8::ROM_START + 1] = 0xCD;
        chip8.memory[Chip8::ROM_START + 2] = 0x12;
        chip8.memory[Chip8::ROM_START + 3] = 0x34;
        assert_eq!(chip8.fetch(), 0xABCD);
        assert_eq!(chip8.fetch(), 0x1234);
    }

    #[test]
    fn test_decode_clear_screen() {
        let mut chip8 = Chip8::new();
        assert_eq!(chip8.decode(0x00E0), Opcode::ClearScreen);
    }

    #[test]
    fn test_decode_jump() {
        let mut chip8 = Chip8::new();
        assert_eq!(chip8.decode(0x1ABC), Opcode::Jump(0xABC));
    }

    #[test]
    fn test_decode_set_register() {
        let mut chip8 = Chip8::new();
        assert_eq!(chip8.decode(0x6A42), Opcode::SetRegister(0xA, 0x42));
    }

    #[test]
    fn test_decode_add() {
        let mut chip8 = Chip8::new();
        assert_eq!(chip8.decode(0x7312), Opcode::Add(0x3, 0x12));
    }

    #[test]
    fn test_decode_set_index_register() {
        let mut chip8 = Chip8::new();
        assert_eq!(chip8.decode(0xA123), Opcode::SetIndexRegister(0x123));
    }

    #[test]
    fn test_decode_draw() {
        let mut chip8 = Chip8::new();
        assert_eq!(chip8.decode(0xD125), Opcode::Draw(0x1, 0x2, 0x5));
    }

    #[test]
    fn test_execute_clear_screen() {
        let mut chip8 = Chip8::new();
        chip8.display[0][0] = true;
        chip8.display[15][32] = true;
        chip8.execute(Opcode::ClearScreen);
        assert_eq!(chip8.display, [[false; 64]; 32]);
    }

    #[test]
    fn test_execute_jump() {
        let mut chip8 = Chip8::new();
        chip8.execute(Opcode::Jump(0x300));
        assert_eq!(chip8.pc, 0x300);
    }

    #[test]
    fn test_execute_set_register() {
        let mut chip8 = Chip8::new();
        chip8.execute(Opcode::SetRegister(0x3, 0x42));
        assert_eq!(chip8.registers[0x3], 0x42);
    }

    #[test]
    fn test_execute_add() {
        let mut chip8 = Chip8::new();
        chip8.registers[0x2] = 0x10;
        chip8.execute(Opcode::Add(0x2, 0x05));
        assert_eq!(chip8.registers[0x2], 0x15);
    }

    #[test]
    fn test_execute_set_index_register() {
        let mut chip8 = Chip8::new();
        chip8.execute(Opcode::SetIndexRegister(0x300));
        assert_eq!(chip8.index, 0x300);
    }

    #[test]
    fn test_execute_draw_sets_pixel() {
        let mut chip8 = Chip8::new();
        // Single row sprite: 0b10000000 = 0x80, draws one pixel at (0,0)
        chip8.memory[0x300] = 0x80;
        chip8.index = 0x300;
        chip8.execute(Opcode::Draw(0, 0, 1));
        assert!(chip8.display[0][0]);
    }

    #[test]
    fn test_execute_draw_collision() {
        let mut chip8 = Chip8::new();
        chip8.memory[0x300] = 0x80;
        chip8.index = 0x300;
        chip8.display[0][0] = true;
        chip8.execute(Opcode::Draw(0, 0, 1));
        assert_eq!(chip8.registers[0xF], 1);
        assert!(!chip8.display[0][0]); // XOR turns it off
    }

    #[test]
    fn test_execute_draw_wraps() {
        let mut chip8 = Chip8::new();
        chip8.memory[0x300] = 0x80;
        chip8.index = 0x300;
        chip8.registers[0] = 63; // vx at edge
        chip8.registers[1] = 0;  // vy at top
        chip8.execute(Opcode::Draw(0, 1, 1));
        assert!(chip8.display[0][63]);
    }
}