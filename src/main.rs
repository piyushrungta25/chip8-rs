use rand::prelude::*;
use std::{thread, time};

type Opcode = u16;

enum Instruction {
    JumpTo(usize),
    Subroutine(usize),

    SkipIfRegisterEqualValue(usize, u8),
    SkipIfRegisterNotEqualValue(usize, u8),

    SkipIfRegisterEqualRegister(usize, usize),

    SetRegisterToValue(usize, u8),
    AddRegisterValue(usize, u8),

    SetRegister(usize, usize),
    SetRegisterOR(usize, usize),
    SetRegisterAND(usize, usize),
    SetRegisterXOR(usize, usize),
    AddRegisterToRegister(usize, usize),
    SubRegisterToRegister(usize, usize),
    ShiftRight(usize),

    ShiftLeft(usize),
    SkipIfRegisterNotEqualRegister(usize, usize),

    SetIndex(usize),

    JumpRelV0(usize),
    RandomAND(usize, u8),

    ClearScreen,
    Return,
    Noop,
}

struct Chip8 {
    memory: Vec<u8>,
    registers: Vec<u8>,
    index: usize, // index register
    pc: usize,    // program counter
    pixel_buffer: Vec<bool>,
    delay_timer: u8,
    sound_timer: u8,

    call_stack: Vec<usize>,
    sp: u8, // stack pointer
    keypad: Vec<u8>,
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            memory: vec![0; 4096],  // 4k memory
            registers: vec![0; 16], // 16 8-bit registers
            index: 0,
            pc: 0x200,                          // program counter starts at 0x200
            pixel_buffer: vec![false; 64 * 32], // 2048 pixels
            delay_timer: 0,
            sound_timer: 0,

            call_stack: vec![0; 16],
            sp: 0,
            keypad: vec![0; 16],
        }
    }

    fn decode(&mut self, oc: Opcode) -> Instruction {
        let reg1: usize = (oc & 0x0F00) as usize;
        let reg2: usize = (oc & 0x00F0) as usize;
        let nnn: usize = (oc & 0x0FFF) as usize;
        let nn: u8 = (oc & 0x00FF) as u8;

        return match oc & 0xF000 {
            0x0000 => match oc & 0x000F {
                0x0000 => Instruction::ClearScreen,
                0x000E => Instruction::Return,
                _ => Instruction::Noop,
            },
            0x1000 => Instruction::JumpTo(nnn),
            0x2000 => Instruction::Subroutine(nnn),

            0x3000 => Instruction::SkipIfRegisterEqualValue(reg1, nn),
            0x4000 => Instruction::SkipIfRegisterNotEqualValue(reg1, nn),

            0x5000 => Instruction::SkipIfRegisterEqualRegister(reg1, reg2),

            0x6000 => Instruction::SetRegisterToValue(reg1, nn),

            0x7000 => Instruction::AddRegisterValue(reg1, nn),

            0x8000 => match oc & 0x000F {
                0x0000 => Instruction::SetRegister(reg1, reg2),
                0x0001 => Instruction::SetRegisterOR(reg1, reg2),
                0x0002 => Instruction::SetRegisterAND(reg1, reg2),
                0x0003 => Instruction::SetRegisterXOR(reg1, reg2),
                0x0004 => Instruction::AddRegisterToRegister(reg1, reg2),
                0x0005 => Instruction::SubRegisterToRegister(reg1, reg2),
                0x0006 => Instruction::ShiftRight(reg1),
                0x0007 => Instruction::SubRegisterToRegister(reg2, reg1),
                0x0008 => Instruction::ShiftLeft(reg1),
                _ => Instruction::Noop,
            },

            0x9000 => Instruction::SkipIfRegisterNotEqualRegister(reg1, reg2),
            0xA000 => Instruction::SetIndex(nnn),
            0xB000 => Instruction::JumpRelV0(nnn),
            0xC000 => Instruction::RandomAND(reg1, nn),

            _ => Instruction::Noop,
        };
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::ClearScreen => {
                self.clear_screen();
                self.pc += 2;
            }
            Instruction::Return => {
                let ret_addr = self.call_stack.pop().unwrap();
                self.pc = ret_addr;
            }
            Instruction::JumpTo(addr) => self.pc = addr,
            Instruction::Subroutine(addr) => {
                self.call_stack.push(self.pc);
                self.pc = addr;
            }
            Instruction::SkipIfRegisterEqualValue(reg, val) => {
                self.pc += 2;
                if self.registers[reg] == val {
                    self.pc += 2;
                }
            }
            Instruction::SkipIfRegisterNotEqualValue(reg, val) => {
                self.pc += 2;
                if self.registers[reg] != val {
                    self.pc += 2;
                }
            }
            Instruction::SkipIfRegisterEqualRegister(reg1, reg2) => {
                self.pc += 2;
                if self.registers[reg1] == self.registers[reg2] {
                    self.pc += 2;
                }
            }
            Instruction::SetRegisterToValue(reg, val) => {
                self.pc += 2;
                self.registers[reg] = val;
            }
            Instruction::AddRegisterValue(reg, val) => {
                self.pc += 2;
                self.registers[reg] = val;
            }
            Instruction::SetRegister(reg1, reg2) => {
                self.pc += 2;
                self.registers[reg1] = self.registers[reg2];
            }
            Instruction::SetRegisterOR(reg1, reg2) => {
                self.pc += 2;
                self.registers[reg1] |= self.registers[reg2];
            }
            Instruction::SetRegisterAND(reg1, reg2) => {
                self.pc += 2;
                self.registers[reg1] &= self.registers[reg2];
            }
            Instruction::SetRegisterXOR(reg1, reg2) => {
                self.pc += 2;
                self.registers[reg1] ^= self.registers[reg2];
            }
            Instruction::AddRegisterToRegister(reg1, reg2) => {
                self.pc += 2;
                let (res, overflow) = self.registers[reg1].overflowing_add(self.registers[reg2]);
                self.registers[reg1] = res;
                self.registers[15] = if overflow { 1 } else { 2 };
            }
            Instruction::SubRegisterToRegister(reg1, reg2) => {
                self.pc += 2;
                let vx = self.registers[reg1];
                let vy = self.registers[reg2];

                self.registers[15] = if vx > vy { 1 } else { 0 };
                self.registers[reg1] = vx.wrapping_sub(vy);
            }
            Instruction::ShiftRight(reg) => {
                self.pc += 2;
                let vx = self.registers[reg];
                self.registers[15] = vx & 0x0001;
                self.registers[reg] = vx >> 1;
            }
            Instruction::ShiftLeft(reg) => {
                self.pc += 2;
                let vx = self.registers[reg];
                self.registers[15] = if (vx & 0x10) > 1 { 1 } else { 0 };
                self.registers[reg] = vx << 1;
            }
            Instruction::SkipIfRegisterNotEqualRegister(reg1, reg2) => {
                self.pc += 2;
                if self.registers[reg1] != self.registers[reg2] {
                    self.pc += 2;
                }
            }
            Instruction::SetIndex(addr) => {
                self.index = addr;
                self.pc += 2;
            }
            Instruction::JumpRelV0(val) => {
                self.pc = val.wrapping_add(self.registers[0] as usize);
            }
            Instruction::RandomAND(reg, val) => {
                let random_byte: u8 = random();
                self.registers[reg] = random_byte & val;
            }

            _ => {}
        }
    }

    fn clear_screen(&mut self) {
        self.pixel_buffer = vec![false; 64 * 32];
    }

    fn fetch(&self) -> Opcode {
        (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16)
    }

    fn update_delay_timer(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    fn update_sound_timer(&mut self) {
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("beep beep!");
            }
            self.sound_timer -= 1;
        }
    }

    fn sleep() {
        thread::sleep(time::Duration::from_millis(16));
    }
}

fn main() {
    let mut c8 = Chip8::new();

    // TODO load font set in memory

    loop {
        let oc = c8.fetch();
        let inst = c8.decode(oc);
        c8.execute(inst);

        c8.update_delay_timer();
        c8.update_sound_timer();

        // we need to run at about 60hz
        Chip8::sleep();
    }
}
