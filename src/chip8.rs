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
    ClearScreen, // 0x00E0
    Return, // 0x00EE
    Jump(u16), // 0x1NNN
    Call(u16), // 0x2NNN
    SkipCondRegEqual(u8,u8), // 0x3XNN
    SkipCondRegNEqual(u8,u8), //0x4XNN
    SkipCondEqual(u8,u8), // 0x5XY0
    SetRegister(u8,u8), // 0x6XNN
    Add(u8,u8), // 0x7XNN
    SkipCondNEqual(u8,u8), // 0x9XY0
    SetIndexRegister(u16), // 0xANNN
    Draw(u8,u8,u8), // 0xDXYN
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

        self.pc += 2;

        opcode
    }

    pub fn decode(&mut self, raw: u16) -> Opcode {
        let first_nibble = (raw & 0xF000) >> 12;

        // TODO: These are wrong - we need the full opcode
        // This is just to get the initial test ROM working
        match first_nibble {
            0x0 => match raw & 0x00FF {
                0xE0 => Opcode::ClearScreen,
                0xEE => Opcode::Return,
                _ => todo!()
            },
            0x1 => Opcode::Jump(raw & 0x0FFF),
            0x2 => Opcode::Call(raw & 0x0FFF),
            0x3 => Opcode::SkipCondRegEqual(((raw & 0x0F00) >> 8) as u8, (raw & 0x00FF) as u8),
            0x4 => Opcode::SkipCondRegNEqual(((raw & 0x0F00) >> 8) as u8, (raw & 0x00FF) as u8),
            0x5 => Opcode::SkipCondEqual(((raw & 0x0F00) >> 8) as u8, ((raw & 0x00F0) >> 4) as u8),
            0x6 => Opcode::SetRegister(((raw & 0x0F00) >> 8) as u8, (raw & 0x00FF) as u8),
            0x7 => Opcode::Add(((raw & 0x0F00) >> 8) as u8, (raw & 0x00FF) as u8),
            0x9 => Opcode::SkipCondNEqual(((raw & 0x0F00) >> 8) as u8, ((raw & 0x00F0) >> 4) as u8),
            0xA => Opcode::SetIndexRegister(raw & 0x0FFF),
            0xD => Opcode::Draw(((raw & 0x0F00) >> 8) as u8, ((raw & 0x00F0) >> 4) as u8, (raw & 0x000F) as u8),

            _ => todo!()
        }
    }

    pub fn display(&self) -> &[[bool; 64]; 32] {
        &self.display
    }

    pub fn execute(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::ClearScreen => self.clear_screen(),
            Opcode::Return => self.ret_subroutine(),
            Opcode::Jump(addr) => self.jump(addr),
            Opcode::Call(addr) => self.call_subroutine(addr),
            Opcode::SkipCondRegEqual(x, nn) => self.skip_if_reg_eq(x, nn),
            Opcode::SkipCondRegNEqual(x, nn) => self.skip_if_reg_neq(x, nn),
            Opcode::SkipCondEqual(x, y) => self.skip_if_eq(x, y),
            Opcode::SetRegister(x, nn) => self.set_register(x, nn),
            Opcode::Add(x, nn) => self.add(x, nn),
            Opcode::SkipCondNEqual(x, y) => self.skip_if_neq(x, y),
            Opcode::SetIndexRegister(addr) => self.set_index_register(addr),
            Opcode::Draw(x, y, n) => self.draw(x, y, n),
        }
    }

    fn clear_screen(&mut self) {
        self.display = [[false;64]; 32];
    }

    fn ret_subroutine(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    fn jump(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn call_subroutine(&mut self, addr: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = addr;
    }

    fn skip_if_reg_eq(&mut self, x: u8, nn: u8) {
        if self.registers[x as usize] == nn {
            self.pc += 2;
        }
    }

    fn skip_if_reg_neq(&mut self, x: u8, nn: u8) {
        if self.registers[x as usize] != nn {
            self.pc += 2;
        }
    }

    fn skip_if_eq(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] == self.registers[y as usize] {
            self.pc += 2
        }
    }

    fn set_register(&mut self, x: u8, nn: u8) {
        self.registers[x as usize] = nn;
    }

    fn skip_if_neq(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] != self.registers[y as usize] {
            self.pc += 2
        }
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
#[path = "chip8_tests.rs"]
mod tests;