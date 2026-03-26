pub struct Chip8 {
    registers: [u8;16],
    memory: [u8;4096],
    index: u16,
    pc: u16,
    stack: [u16;16],
    sp: u8,
}