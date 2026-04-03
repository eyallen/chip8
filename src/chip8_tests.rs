use super::*;
use std::io::Write;
use tempfile::NamedTempFile;

// --- new / init ---

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

// --- load_rom ---

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

// --- fetch ---

#[test]
fn test_fetch_returns_opcode() {
    let mut chip8 = Chip8::new();
    chip8.memory[Chip8::ROM_START] = 0xD1;
    chip8.memory[Chip8::ROM_START + 1] = 0x23;
    assert_eq!(chip8.fetch(), 0xD123);
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

// --- decode ---

#[test]
fn test_decode_clear_screen() {
    let mut chip8 = Chip8::new();
    assert_eq!(chip8.decode(0x00E0), Opcode::ClearScreen);
}

#[test]
fn test_decode_return() {
    let mut chip8 = Chip8::new();
    assert_eq!(chip8.decode(0x00EE), Opcode::Return);
}

#[test]
fn test_decode_jump() {
    let mut chip8 = Chip8::new();
    assert_eq!(chip8.decode(0x1ABC), Opcode::Jump(0xABC));
}

#[test]
fn test_decode_call() {
    let mut chip8 = Chip8::new();
    assert_eq!(chip8.decode(0x2ABC), Opcode::Call(0xABC));
}

#[test]
fn test_decode_skip_cond_reg_equal() {
    let mut chip8 = Chip8::new();
    assert_eq!(chip8.decode(0x3A42), Opcode::SkipCondRegEqual(0xA, 0x42));
}

#[test]
fn test_decode_skip_cond_reg_nequal() {
    let mut chip8 = Chip8::new();
    assert_eq!(chip8.decode(0x4A42), Opcode::SkipCondRegNEqual(0xA, 0x42));
}

#[test]
fn test_decode_skip_cond_equal() {
    let mut chip8 = Chip8::new();
    assert_eq!(chip8.decode(0x5AB0), Opcode::SkipCondEqual(0xA, 0xB));
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
fn test_decode_skip_cond_nequal() {
    let mut chip8 = Chip8::new();
    assert_eq!(chip8.decode(0x9AB0), Opcode::SkipCondNEqual(0xA, 0xB));
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

// --- execute ---

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
fn test_execute_call_and_return() {
    let mut chip8 = Chip8::new();
    let start_pc = chip8.pc;
    chip8.execute(Opcode::Call(0x300));
    assert_eq!(chip8.pc, 0x300);
    assert_eq!(chip8.stack[0], start_pc);
    assert_eq!(chip8.sp, 1);
    chip8.execute(Opcode::Return);
    assert_eq!(chip8.pc, start_pc);
    assert_eq!(chip8.sp, 0);
}

#[test]
fn test_execute_skip_cond_reg_equal_skips() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0x42;
    let pc_before = chip8.pc;
    chip8.execute(Opcode::SkipCondRegEqual(0x1, 0x42));
    assert_eq!(chip8.pc, pc_before + 2);
}

#[test]
fn test_execute_skip_cond_reg_equal_no_skip() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0x10;
    let pc_before = chip8.pc;
    chip8.execute(Opcode::SkipCondRegEqual(0x1, 0x42));
    assert_eq!(chip8.pc, pc_before);
}

#[test]
fn test_execute_skip_cond_reg_nequal_skips() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0x10;
    let pc_before = chip8.pc;
    chip8.execute(Opcode::SkipCondRegNEqual(0x1, 0x42));
    assert_eq!(chip8.pc, pc_before + 2);
}

#[test]
fn test_execute_skip_cond_equal_skips() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0x42;
    chip8.registers[0x2] = 0x42;
    let pc_before = chip8.pc;
    chip8.execute(Opcode::SkipCondEqual(0x1, 0x2));
    assert_eq!(chip8.pc, pc_before + 2);
}

#[test]
fn test_execute_skip_cond_nequal_skips() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0x10;
    chip8.registers[0x2] = 0x42;
    let pc_before = chip8.pc;
    chip8.execute(Opcode::SkipCondNEqual(0x1, 0x2));
    assert_eq!(chip8.pc, pc_before + 2);
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
    assert!(!chip8.display[0][0]);
}

#[test]
fn test_execute_draw_wraps() {
    let mut chip8 = Chip8::new();
    chip8.memory[0x300] = 0x80;
    chip8.index = 0x300;
    chip8.registers[0] = 63;
    chip8.registers[1] = 0;
    chip8.execute(Opcode::Draw(0, 1, 1));
    assert!(chip8.display[0][63]);
}

// --- 8XY arithmetic ---

#[test]
fn test_execute_set() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x2] = 0xAB;
    chip8.execute(Opcode::Set(0x1, 0x2));
    assert_eq!(chip8.registers[0x1], 0xAB);
}

#[test]
fn test_execute_or() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0b1010;
    chip8.registers[0x2] = 0b0101;
    chip8.execute(Opcode::Or(0x1, 0x2));
    assert_eq!(chip8.registers[0x1], 0b1111);
}

#[test]
fn test_execute_and() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0b1110;
    chip8.registers[0x2] = 0b0111;
    chip8.execute(Opcode::And(0x1, 0x2));
    assert_eq!(chip8.registers[0x1], 0b0110);
}

#[test]
fn test_execute_xor() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0b1010;
    chip8.registers[0x2] = 0b1100;
    chip8.execute(Opcode::Xor(0x1, 0x2));
    assert_eq!(chip8.registers[0x1], 0b0110);
}

#[test]
fn test_execute_add_overflow_no_carry() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 10;
    chip8.registers[0x2] = 20;
    chip8.execute(Opcode::AddOverflow(0x1, 0x2));
    assert_eq!(chip8.registers[0x1], 30);
    assert_eq!(chip8.registers[0xF], 0);
}

#[test]
fn test_execute_add_overflow_carry() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 200;
    chip8.registers[0x2] = 100;
    chip8.execute(Opcode::AddOverflow(0x1, 0x2));
    assert_eq!(chip8.registers[0x1], 44); // 300 % 256
    assert_eq!(chip8.registers[0xF], 1);
}

#[test]
fn test_execute_sub_no_borrow() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 10;
    chip8.registers[0x2] = 3;
    chip8.execute(Opcode::Sub(0x1, 0x2));
    assert_eq!(chip8.registers[0x1], 7);
    assert_eq!(chip8.registers[0xF], 1); // no borrow
}

#[test]
fn test_execute_sub_borrow() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 3;
    chip8.registers[0x2] = 10;
    chip8.execute(Opcode::Sub(0x1, 0x2));
    assert_eq!(chip8.registers[0xF], 0); // borrow occurred
}

#[test]
fn test_execute_sub_n_no_borrow() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 3;
    chip8.registers[0x2] = 10;
    chip8.execute(Opcode::SubN(0x1, 0x2));
    assert_eq!(chip8.registers[0x1], 7); // VY - VX
    assert_eq!(chip8.registers[0xF], 1);
}

#[test]
fn test_execute_sub_n_borrow() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 10;
    chip8.registers[0x2] = 3;
    chip8.execute(Opcode::SubN(0x1, 0x2));
    assert_eq!(chip8.registers[0xF], 0);
}

#[test]
fn test_execute_shift_right_lsb_set() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0b00000101;
    chip8.execute(Opcode::ShiftRight(0x1, 0x0));
    assert_eq!(chip8.registers[0x1], 0b00000010);
    assert_eq!(chip8.registers[0xF], 1);
}

#[test]
fn test_execute_shift_right_lsb_clear() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0b00000100;
    chip8.execute(Opcode::ShiftRight(0x1, 0x0));
    assert_eq!(chip8.registers[0x1], 0b00000010);
    assert_eq!(chip8.registers[0xF], 0);
}

#[test]
fn test_execute_shift_left_msb_set() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0b10000001;
    chip8.execute(Opcode::ShiftLeft(0x1, 0x0));
    assert_eq!(chip8.registers[0x1], 0b00000010);
    assert_eq!(chip8.registers[0xF], 1);
}

#[test]
fn test_execute_shift_left_msb_clear() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0b00000001;
    chip8.execute(Opcode::ShiftLeft(0x1, 0x0));
    assert_eq!(chip8.registers[0x1], 0b00000010);
    assert_eq!(chip8.registers[0xF], 0);
}

// --- jump_v0, random ---

#[test]
fn test_execute_jump_v0() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x0] = 0x10;
    chip8.execute(Opcode::JumpV0(0x200));
    assert_eq!(chip8.pc, 0x210);
}

#[test]
fn test_execute_random_masked() {
    // Can't test the random value, but can confirm masking: with mask=0 result must be 0
    let mut chip8 = Chip8::new();
    chip8.execute(Opcode::Random(0x1, 0x00));
    assert_eq!(chip8.registers[0x1], 0);
}

// --- key opcodes ---

#[test]
fn test_execute_skip_if_key_pressed() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0x5;
    chip8.keys[0x5] = true;
    let pc_before = chip8.pc;
    chip8.execute(Opcode::SkipIfKey(0x1));
    assert_eq!(chip8.pc, pc_before + 2);
}

#[test]
fn test_execute_skip_if_key_not_pressed() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0x5;
    chip8.keys[0x5] = false;
    let pc_before = chip8.pc;
    chip8.execute(Opcode::SkipIfKey(0x1));
    assert_eq!(chip8.pc, pc_before);
}

#[test]
fn test_execute_skip_if_not_key_not_pressed() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0x5;
    chip8.keys[0x5] = false;
    let pc_before = chip8.pc;
    chip8.execute(Opcode::SkipIfNotKey(0x1));
    assert_eq!(chip8.pc, pc_before + 2);
}

#[test]
fn test_execute_skip_if_not_key_pressed() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0x5;
    chip8.keys[0x5] = true;
    let pc_before = chip8.pc;
    chip8.execute(Opcode::SkipIfNotKey(0x1));
    assert_eq!(chip8.pc, pc_before);
}

#[test]
fn test_execute_wait_for_key_found() {
    let mut chip8 = Chip8::new();
    chip8.keys[0x7] = true;
    let pc_before = chip8.pc;
    chip8.execute(Opcode::WaitForKey(0x1));
    assert_eq!(chip8.registers[0x1], 0x7);
    assert_eq!(chip8.pc, pc_before);
}

#[test]
fn test_execute_wait_for_key_not_found() {
    let mut chip8 = Chip8::new();
    let pc_before = chip8.pc;
    chip8.execute(Opcode::WaitForKey(0x1));
    assert_eq!(chip8.pc, pc_before - 2); // re-executes next cycle
}

// --- timers ---

#[test]
fn test_execute_get_delay_timer() {
    let mut chip8 = Chip8::new();
    chip8.delay_timer = 42;
    chip8.execute(Opcode::GetDelayTimer(0x1));
    assert_eq!(chip8.registers[0x1], 42);
}

#[test]
fn test_execute_set_delay_timer() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 30;
    chip8.execute(Opcode::SetDelayTimer(0x1));
    assert_eq!(chip8.delay_timer, 30);
}

#[test]
fn test_execute_set_sound_timer() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 10;
    chip8.execute(Opcode::SetSoundTimer(0x1));
    assert_eq!(chip8.sound_timer, 10);
}

#[test]
fn test_tick_timers_decrements() {
    let mut chip8 = Chip8::new();
    chip8.delay_timer = 5;
    chip8.sound_timer = 3;
    chip8.tick_timers();
    assert_eq!(chip8.delay_timer, 4);
    assert_eq!(chip8.sound_timer, 2);
}

#[test]
fn test_tick_timers_no_underflow() {
    let mut chip8 = Chip8::new();
    chip8.delay_timer = 0;
    chip8.sound_timer = 0;
    chip8.tick_timers();
    assert_eq!(chip8.delay_timer, 0);
    assert_eq!(chip8.sound_timer, 0);
}

// --- index / memory ops ---

#[test]
fn test_execute_add_to_index() {
    let mut chip8 = Chip8::new();
    chip8.index = 0x300;
    chip8.registers[0x1] = 0x10;
    chip8.execute(Opcode::AddToIndex(0x1));
    assert_eq!(chip8.index, 0x310);
}

#[test]
fn test_execute_font_char() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 0x3; // digit 3
    chip8.execute(Opcode::FontChar(0x1));
    assert_eq!(chip8.index, Chip8::FONT_START as u16 + 3 * 5);
}

#[test]
fn test_execute_bcd() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x1] = 123;
    chip8.index = 0x300;
    chip8.execute(Opcode::BCD(0x1));
    assert_eq!(chip8.memory[0x300], 1);
    assert_eq!(chip8.memory[0x301], 2);
    assert_eq!(chip8.memory[0x302], 3);
}

#[test]
fn test_execute_store_registers() {
    let mut chip8 = Chip8::new();
    chip8.registers[0x0] = 0xAA;
    chip8.registers[0x1] = 0xBB;
    chip8.registers[0x2] = 0xCC;
    chip8.index = 0x300;
    chip8.execute(Opcode::StoreRegisters(0x2));
    assert_eq!(chip8.memory[0x300], 0xAA);
    assert_eq!(chip8.memory[0x301], 0xBB);
    assert_eq!(chip8.memory[0x302], 0xCC);
}

#[test]
fn test_execute_load_registers() {
    let mut chip8 = Chip8::new();
    chip8.memory[0x300] = 0x11;
    chip8.memory[0x301] = 0x22;
    chip8.memory[0x302] = 0x33;
    chip8.index = 0x300;
    chip8.execute(Opcode::LoadRegisters(0x2));
    assert_eq!(chip8.registers[0x0], 0x11);
    assert_eq!(chip8.registers[0x1], 0x22);
    assert_eq!(chip8.registers[0x2], 0x33);
}
