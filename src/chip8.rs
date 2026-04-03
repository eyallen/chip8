use std::fs::metadata;
use rand::Rng;

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
    keys: [bool; 16]
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
    Set(u8,u8), // 0x8XY0
    Or(u8,u8), // 0x8XY1
    And(u8,u8), // 0x8XY2
    Xor(u8,u8), // 0x8XY3
    AddOverflow(u8,u8), //0x8XY4
    Sub(u8,u8), // 0x8XY5
    ShiftRight(u8,u8), // 0x8XY6
    SubN(u8,u8), // 0x8XY7
    ShiftLeft(u8,u8), // 0x8XYE
    SkipCondNEqual(u8,u8), // 0x9XY0
    SetIndexRegister(u16), // 0xANNN
    JumpV0(u16), // 0xBNNN
    Random(u8,u8), // 0xCXNN
    Draw(u8,u8,u8), // 0xDXYN
    SkipIfKey(u8), // 0xEX9E
    SkipIfNotKey(u8), // 0xEXA1
    GetDelayTimer(u8), // 0xFX07
    WaitForKey(u8), // 0xFX0A
    SetDelayTimer(u8), // 0xFX15
    SetSoundTimer(u8), // 0xFX18
    AddToIndex(u8), // 0xFX1E
    FontChar(u8), // 0xFX29
    BCD(u8), // 0xFX33
    StoreRegisters(u8), // 0xFX55
    LoadRegisters(u8), // 0xFX65
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
        let mut chip8 = Chip8 { registers: [0;16], memory: [0;4096], index: 0, pc: Self::ROM_START as u16, stack: [0;16], sp: 0, delay_timer: 0, sound_timer: 0, display: [[false;64]; 32], keys: [false; 16] };


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
            0x8 => match raw & 0x000F {
                0x0 => Opcode::Set(((raw & 0x0F00) >> 8) as u8, ((raw & 0x00F0) >> 4) as u8),
                0x1 => Opcode::Or(((raw & 0x0F00) >> 8) as u8, ((raw & 0x00F0) >> 4) as u8),
                0x2 => Opcode::And(((raw & 0x0F00) >> 8) as u8, ((raw & 0x00F0) >> 4) as u8),
                0x3 => Opcode::Xor(((raw & 0x0F00) >> 8) as u8, ((raw & 0x00F0) >> 4) as u8),
                0x4 => Opcode::AddOverflow(((raw & 0x0F00) >> 8) as u8, ((raw & 0x00F0) >> 4) as u8),
                0x5 => Opcode::Sub(((raw & 0x0F00) >> 8) as u8, ((raw & 0x00F0) >> 4) as u8),
                0x6 => Opcode::ShiftRight(((raw & 0x0F00) >> 8) as u8, ((raw & 0x00F0) >> 4) as u8),
                0x7 => Opcode::SubN(((raw & 0x0F00) >> 8) as u8, ((raw & 0x00F0) >> 4) as u8),
                0xE => Opcode::ShiftLeft(((raw & 0x0F00) >> 8) as u8, ((raw & 0x00F0) >> 4) as u8),
                _ => todo!()
            }
            0x9 => Opcode::SkipCondNEqual(((raw & 0x0F00) >> 8) as u8, ((raw & 0x00F0) >> 4) as u8),
            0xA => Opcode::SetIndexRegister(raw & 0x0FFF),
            0xB => Opcode::JumpV0(raw & 0x0FFF),
            0xC => Opcode::Random(((raw & 0x0F00) >> 8) as u8, (raw & 0x00FF) as u8),
            0xD => Opcode::Draw(((raw & 0x0F00) >> 8) as u8, ((raw & 0x00F0) >> 4) as u8, (raw & 0x000F) as u8),
            0xE => match raw & 0x00FF {
                0x9E => Opcode::SkipIfKey(((raw & 0x0F00) >> 8) as u8),
                0xA1 => Opcode::SkipIfNotKey(((raw & 0x0F00) >> 8) as u8),
                _ => todo!()
            },
            0xF => match raw & 0x00FF {
                0x07 => Opcode::GetDelayTimer(((raw & 0x0F00) >> 8) as u8),
                0x0A => Opcode::WaitForKey(((raw & 0x0F00) >> 8) as u8),
                0x15 => Opcode::SetDelayTimer(((raw & 0x0F00) >> 8) as u8),
                0x18 => Opcode::SetSoundTimer(((raw & 0x0F00) >> 8) as u8),
                0x1E => Opcode::AddToIndex(((raw & 0x0F00) >> 8) as u8),
                0x29 => Opcode::FontChar(((raw & 0x0F00) >> 8) as u8),
                0x33 => Opcode::BCD(((raw & 0x0F00) >> 8) as u8),
                0x55 => Opcode::StoreRegisters(((raw & 0x0F00) >> 8) as u8),
                0x65 => Opcode::LoadRegisters(((raw & 0x0F00) >> 8) as u8),
                _ => todo!()
            },
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
            Opcode::Set(x, y) => self.set(x, y),
            Opcode::Or(x, y) => self.or(x, y),
            Opcode::And(x, y) => self.and(x, y),
            Opcode::Xor(x, y) => self.xor(x, y),
            Opcode::AddOverflow(x, y) => self.add_overflow(x, y),
            Opcode::Sub(x, y) => self.sub(x, y),
            Opcode::ShiftRight(x, y) => self.shift_right(x, y),
            Opcode::SubN(x, y) => self.sub_n(x, y),
            Opcode::ShiftLeft(x, y) => self.shift_left(x, y),
            Opcode::SkipCondNEqual(x, y) => self.skip_if_neq(x, y),
            Opcode::SetIndexRegister(addr) => self.set_index_register(addr),
            Opcode::JumpV0(addr) => self.jump_v0(addr),
            Opcode::Random(x, nn) => self.random(x, nn),
            Opcode::Draw(x, y, n) => self.draw(x, y, n),
            Opcode::SkipIfKey(x) => self.skip_if_key(x),
            Opcode::SkipIfNotKey(x) => self.skip_if_not_key(x),
            Opcode::GetDelayTimer(x) => self.get_delay_timer(x),
            Opcode::WaitForKey(x) => self.wait_for_key(x),
            Opcode::SetDelayTimer(x) => self.set_delay_timer(x),
            Opcode::SetSoundTimer(x) => self.set_sound_timer(x),
            Opcode::AddToIndex(x) => self.add_to_index(x),
            Opcode::FontChar(x) => self.font_char(x),
            Opcode::BCD(x) => self.bcd(x),
            Opcode::StoreRegisters(x) => self.store_registers(x),
            Opcode::LoadRegisters(x) => self.load_registers(x),
        }
    }

    pub fn set_key(&mut self, key: usize, pressed: bool) {
        self.keys[key] = pressed;
    }

    fn clear_screen(&mut self) { // 0x00E0
        self.display = [[false;64]; 32];
    }

    fn ret_subroutine(&mut self) { // 0x00EE
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    fn jump(&mut self, addr: u16) { // 0x1NNN
        self.pc = addr;
    }

    fn call_subroutine(&mut self, addr: u16) { // 0x2NNN
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = addr;
    }

    fn skip_if_reg_eq(&mut self, x: u8, nn: u8) { // 0x3XNN
        if self.registers[x as usize] == nn {
            self.pc += 2;
        }
    }

    fn skip_if_reg_neq(&mut self, x: u8, nn: u8) { // 0x4XNN
        if self.registers[x as usize] != nn {
            self.pc += 2;
        }
    }

    fn skip_if_eq(&mut self, x: u8, y: u8) { // 0x5XY0
        if self.registers[x as usize] == self.registers[y as usize] {
            self.pc += 2
        }
    }

    fn set_register(&mut self, x: u8, nn: u8) { // 0x6XNN
        self.registers[x as usize] = nn;
    }

    fn skip_if_neq(&mut self, x: u8, y: u8) { // 0x9XY0
        if self.registers[x as usize] != self.registers[y as usize] {
            self.pc += 2
        }
    }

    fn add(&mut self, x: u8, nn: u8) { // 0x7XNN
        // Chip 8 wraps on overflow. To avoid a panic, we need wrapping_add
        self.registers[x as usize] = self.registers[x as usize].wrapping_add(nn);
    }

    fn set(&mut self, x: u8, y: u8) { // 0x8XY0
        self.registers[x as usize] = self.registers[y as usize];
    }

    fn or(&mut self, x: u8, y: u8) { // 0x8XY1
        self.registers[x as usize] = self.registers[x as usize] | self.registers[y as usize];
    }

    fn and(&mut self, x: u8, y: u8) { // 0x8XY2
        self.registers[x as usize] = self.registers[x as usize] & self.registers[y as usize];
    }

    fn xor(&mut self, x: u8, y: u8) { // 0x8XY3
        self.registers[x as usize] = self.registers[x as usize] ^ self.registers[y as usize];
    }

    fn add_overflow(&mut self, x: u8, y: u8) { // 0x8XY4
        let (result, overflowed) = self.registers[x as usize].overflowing_add(self.registers[y as usize]);
        self.registers[0xF] = overflowed as u8; // 1 if carry, 0 if not
        self.registers[x as usize] = result;
    }

    fn sub(&mut self, x: u8, y: u8) { // 0x8XY5
        let (result, underflowed) = self.registers[x as usize].overflowing_sub(self.registers[y as usize]);                                                   
        self.registers[0xF] = !underflowed as u8; // VF = 1 if NO borrow                                                                                      
        self.registers[x as usize] = result;
    }

    fn shift_right(&mut self, x: u8, y: u8) { // 0x8XY6
        self.registers[0xF] = self.registers[x as usize] & 0x1;                                                                                                   
        self.registers[x as usize] >>= 1;
    }

    fn sub_n(&mut self, x: u8, y: u8) { // 0x8XY7
        let (result, underflowed) = self.registers[y as usize].overflowing_sub(self.registers[x as usize]);                                                   
        self.registers[0xF] = !underflowed as u8;                                                                                                             
        self.registers[x as usize] = result;
    }

    fn shift_left(&mut self, x: u8, y: u8) { // 0x8XYE
        self.registers[0xF] = (self.registers[x as usize] >> 7) & 0x1;                                                                                            
        self.registers[x as usize] <<= 1; 
    }

    fn jump_v0(&mut self, addr: u16) { // 0xBNNN
        self.pc = (self.registers[0x0] as u16) + addr;
    }

    fn random(&mut self, x: u8, nn: u8) { // 0xCXNN
        self.registers[x as usize] = rand::rng().random::<u8>() & nn;
    }

    fn skip_if_key(&mut self, x: u8) { // 0xEX9E
        let key = self.registers[x as usize];
        if self.keys[key as usize] {
            self.pc += 2;
        }
    }

    fn skip_if_not_key(&mut self, x: u8) { // 0xEXA1
        let key = self.registers[x as usize];
        if !self.keys[key as usize] {
            self.pc += 2;
        }
    }

    fn get_delay_timer(&mut self, x: u8) { // 0xFX07
        self.registers[x as usize] = self.delay_timer;
    }

    fn wait_for_key(&mut self, x: u8) { // 0xFX0A
        if let Some(key) = self.keys.iter().position(|&k| k) {
            self.registers[x as usize] = key as u8;
        } else {
            self.pc -= 2; // retry next cycle
        }
    }

    fn set_delay_timer(&mut self, x: u8) { // 0xFX15
        self.delay_timer = self.registers[x as usize];
    }

    fn set_sound_timer(&mut self, x: u8) { // 0xFX18
        self.sound_timer = self.registers[x as usize];
    }

    fn add_to_index(&mut self, x: u8) { // 0xFX1E
        self.index += self.registers[x as usize] as u16;
    }

    fn font_char(&mut self, x: u8) { // 0xFX29
        self.index = (Self::FONT_START as u16) + (self.registers[x as usize] as u16 * 5);
    }

    fn bcd(&mut self, x: u8) { // 0xFX33
        let vx = self.registers[x as usize];
        self.memory[self.index as usize] = vx / 100;
        self.memory[self.index as usize + 1] = (vx/10) % 10;
        self.memory[self.index as usize + 2] = vx % 10;
    }

    fn store_registers(&mut self, x: u8) { // 0xFX55
        for i in 0..=x {
            self.memory[(self.index + i as u16) as usize] = self.registers[i as usize];
        }
    }

    fn load_registers(&mut self, x: u8) { // 0xFX65
        for i in 0..=x {
            self.registers[i as usize] = self.memory[(self.index + i as u16) as usize];
        }
    }

    fn set_index_register(&mut self, addr: u16) { // 0xANNN
        self.index = addr;
    }

    fn draw(&mut self, x: u8, y: u8, n: u8) { // 0xDXYN
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