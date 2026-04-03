---
name: CHIP-8 emulator project
description: Context about the chip8 emulator learning project in Rust
type: project
---

Building a CHIP-8 emulator in Rust as a learning project. Terminal-based renderer using block characters. ROM path passed as CLI arg.

**Why:** Learning project — user is using it to practice Rust and learn emulator concepts (e.g. manually writing bit-shifting for opcode decoding).

**How to apply:** Keep explanations educational. Let the user do hands-on work (bit shifting, impl details) and scaffold/structure for them when asked.

Key implementation notes:
- `0x7XNN` ADD wraps on overflow (wrapping_add) — no carry flag set, per spec
- `0x8XY_` opcodes are partially stubbed in decode with `todo!()` placeholders for the user to fill in bit shifting
- VS Code debug config at `.vscode/launch.json` uses CodeLLDB, one config per ROM in `roms/`
