use std::{thread, time};

type Opcode = u16;

enum Instruction {
    SetI(u16),
    Noop,
}

struct Chip8 {
    memory: Vec<u8>,
    registers: Vec<u8>,
    index: u16, // index register
    pc: usize,  // program counter
    pixel_buffer: Vec<bool>,
    delay_timer: u8,
    sound_timer: u8,

    stack: Vec<u16>,
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

            stack: vec![0; 16],
            sp: 0,
            keypad: vec![0; 16],
        }
    }

    fn fetch(&self) -> Opcode {
        (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16)
    }

    fn decode(&mut self, oc: Opcode) -> Instruction {
        return match oc & 0xF000 {
            0xA000 => Instruction::SetI(oc & 0x0FFF),
            _ => Instruction::Noop,
        };
    }

    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::SetI(addr) => {
                self.index = addr;
                self.pc += 2;
            }
            _ => {}
        }
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
