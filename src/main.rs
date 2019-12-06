extern crate sdl2;

use rand::prelude::*;
use std::{thread, time};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::fs::File;
use std::io::Read;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

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
    SubRegisterToRegister85(usize, usize),
    SubRegisterToRegister87(usize, usize),
    ShiftRight(usize),

    ShiftLeft(usize),
    SkipIfRegisterNotEqualRegister(usize, usize),

    SetIndex(usize),

    JumpRelV0(usize),
    RandomAND(usize, u8),
    Draw(usize, usize, u8),

    SkipIfKey(usize),
    SkipIfNotKey(usize),

    SetToDelayTimer(usize),
    GetKeyPress(usize),
    SetDelayTimer(usize),
    SetSoundTimer(usize),
    AddToIndexRegister(usize),
    SetIndexToSpriteAddr(usize),
    BCD(usize),
    DumpRegistersTill(usize),
    LoadRegistersTill(usize),

    ClearScreen,
    Return,
    Noop,
}

struct Chip8 {
    memory: Vec<u8>,
    registers: Vec<u8>,
    index: usize, // index register
    pc: usize,    // program counter
    pixel_buffer: Vec<Vec<bool>>,
    delay_timer: u8,
    sound_timer: u8,

    call_stack: Vec<usize>,
    keypad: Vec<bool>,

    canvas: Canvas<Window>,
}

impl Chip8 {
    fn new(canvas: Canvas<Window>) -> Self {
        let mut c8 = Chip8 {
            memory: vec![0; 4096],  // 4k memory
            registers: vec![0; 16], // 16 8-bit registers
            index: 0,
            pc: 0x200,                               // program counter starts at 0x200
            pixel_buffer: vec![vec![false; 64]; 32], // 2048 pixels
            delay_timer: 0,
            sound_timer: 0,

            call_stack: vec![0; 16],
            keypad: vec![false; 16],
            canvas
        };

        c8.load_fonts();
        c8
    }

    fn load_fonts(&mut self) {
        let chip8_fontset: [u8; 80] = [
            0b11110000, 0b10010000, 0b10010000, 0b10010000, 0b11110000, // Zero
            0b00100000, 0b01100000, 0b00100000, 0b00100000, 0b01110000, // One
            0b11110000, 0b00010000, 0b11110000, 0b10000000, 0b11110000, // Two
            0b11110000, 0b00010000, 0b11110000, 0b00010000, 0b11110000, // Three
            0b10010000, 0b10010000, 0b11110000, 0b00010000, 0b00010000, // Four
            0b11110000, 0b10000000, 0b11110000, 0b00010000, 0b11110000, // Five
            0b11110000, 0b10000000, 0b11110000, 0b10010000, 0b11110000, // Six
            0b11110000, 0b00010000, 0b00100000, 0b01000000, 0b01000000, // Seven
            0b11110000, 0b10010000, 0b11110000, 0b10010000, 0b11110000, // Eight
            0b11110000, 0b10010000, 0b11110000, 0b00010000, 0b11110000, // Nine
            0b11110000, 0b10010000, 0b11110000, 0b10010000, 0b10010000, // A
            0b11100000, 0b10010000, 0b11100000, 0b10010000, 0b11100000, // B
            0b11110000, 0b10000000, 0b10000000, 0b10000000, 0b11110000, // C
            0b11100000, 0b10010000, 0b10010000, 0b10010000, 0b11100000, // D
            0b11110000, 0b10000000, 0b11110000, 0b10000000, 0b11110000, // E
            0b11110000, 0b10000000, 0b11110000, 0b10000000, 0b10000000, // F
        ];

        self.memory[0x50..0xA0].copy_from_slice(&chip8_fontset);
    }

    fn decode(&mut self, oc: Opcode) -> Instruction {
        let reg1: usize = ((oc & 0x0F00) >> 8) as usize;
        let reg2: usize = ((oc & 0x00F0) >> 4) as usize;
        let nnn: usize = (oc & 0x0FFF) as usize;
        let nn: u8 = (oc & 0x00FF) as u8;
        let n: u8 = (oc & 0x000F) as u8;


        return match oc & 0xF000 {
            0x0000 => match oc & 0x00FF {
                0x00E0 => Instruction::ClearScreen,
                0x00EE => Instruction::Return,
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
                0x0005 => Instruction::SubRegisterToRegister85(reg1, reg2),
                0x0006 => Instruction::ShiftRight(reg1),
                0x0007 => Instruction::SubRegisterToRegister87(reg1, reg2),
                0x000E => Instruction::ShiftLeft(reg1),
                _ => Instruction::Noop,
            },

            0x9000 => Instruction::SkipIfRegisterNotEqualRegister(reg1, reg2),
            0xA000 => Instruction::SetIndex(nnn),
            0xB000 => Instruction::JumpRelV0(nnn),
            0xC000 => Instruction::RandomAND(reg1, nn),

            0xD000 => Instruction::Draw(reg1, reg2, n),

            0xE000 => match oc & 0x00FF {
                0x009E => Instruction::SkipIfKey(reg1),
                0x00A1 => Instruction::SkipIfNotKey(reg1),
                _ => Instruction::Noop,
            },

            0xF000 => match oc & 0x00FF {
                0x0007 => Instruction::SetToDelayTimer(reg1),
                0x000A => Instruction::GetKeyPress(reg1),
                0x0015 => Instruction::SetDelayTimer(reg1),
                0x0018 => Instruction::SetSoundTimer(reg1),
                0x001E => Instruction::AddToIndexRegister(reg1),
                0x0029 => Instruction::SetIndexToSpriteAddr(reg1),
                0x0033 => Instruction::BCD(reg1),
                0x0055 => Instruction::DumpRegistersTill(reg1),
                0x0065 => Instruction::LoadRegistersTill(reg1),
                _ => Instruction::Noop,
            },

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
                self.call_stack.push(self.pc+2);
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
                self.registers[reg] = self.registers[reg].wrapping_add(val);
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
                self.registers[15] = if overflow { 1 } else { 0 };
            }
            Instruction::SubRegisterToRegister85(reg1, reg2) => {
                self.pc += 2;
                let vx = self.registers[reg1];
                let vy = self.registers[reg2];

                self.registers[15] = if vx > vy { 1 } else { 0 };
                self.registers[reg1] = vx.wrapping_sub(vy);
            }
            Instruction::SubRegisterToRegister87(reg1, reg2) => {
                self.pc += 2;
                let vx = self.registers[reg1];
                let vy = self.registers[reg2];

                self.registers[15] = if vx < vy { 1 } else { 0 };
                self.registers[reg1] = vy.wrapping_sub(vx);
            }
            Instruction::ShiftRight(reg) => {
                self.pc += 2;
                let vx = self.registers[reg];
                self.registers[15] = vx & 1;
                self.registers[reg] = vx >> 1;
            }
            Instruction::ShiftLeft(reg) => {
                self.pc += 2;
                let vx = self.registers[reg];
                self.registers[15] = (vx & 0b10000000) >> 7;
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
                self.pc += 2;
                let random_byte: u8 = random();
                self.registers[reg] = random_byte & val;
            }
            Instruction::Draw(reg1, reg2, height) => {
                self.pc += 2;
                let x = self.registers[reg1] as usize;
                let y = self.registers[reg2] as usize;

                let mut did_overflow: bool = false;
                for i in 0usize..(height as usize) {
                    let word = self.memory[self.index + i];
                    for j in 0usize..8 {
                        let new_val: bool  = ( (word >> (7-j)) & 1) != 0;
                        let tx = (x + j)%64;
                        let ty = (y + i)%32;
                        let cur_val = self.pixel_buffer[ty][tx];
                        if cur_val==true && new_val==false {
                            did_overflow = true;
                        }
                        if new_val {
                            self.pixel_buffer[ty][tx] = !cur_val;

                        }

                    }
                }

                self.registers[15] = if did_overflow { 1 } else { 0 };
                self.rerender();
            }
            Instruction::SkipIfKey(reg) => {
                self.pc += 2;  // TODO
                // if self.keypad[self.registers[reg] as usize] {
                //     self.pc += 2;
                // }
            }
            Instruction::SkipIfNotKey(reg) => {
                self.pc += 2;  // TODO
                // if !self.keypad[self.registers[reg] as usize] {
                //     self.pc += 2;
                // }
            }
            Instruction::SetToDelayTimer(reg) => {
                self.pc += 2;
                self.registers[reg] = self.delay_timer;
            }
            Instruction::GetKeyPress(reg) => {
                self.pc += 2;  // TODO
                // self.registers[reg] = self.get_key_press();
            }
            Instruction::SetDelayTimer(reg) => {
                self.pc += 2;
                self.delay_timer = self.registers[reg];
            }
            Instruction::SetSoundTimer(reg) => {
                self.pc += 2;
                self.sound_timer = self.registers[reg];
            }
            Instruction::AddToIndexRegister(reg) => {
                self.pc += 2;
                self.index += (self.registers[reg] as usize);
                self.registers[15] = if self.index > 0x0F00 { 1 } else { 0 };
            }
            Instruction::SetIndexToSpriteAddr(reg) => {
                self.pc += 2;
                let vx = self.registers[reg];
                self.index = 0x50 + (5 * vx as usize);
            }
            Instruction::BCD(reg) => {
                self.pc += 2;
                let vx = self.registers[reg];
                self.memory[self.index] = vx/100;
                self.memory[self.index + 1] = (vx/10)%10;
                self.memory[self.index + 2] = vx%10;
            }
            Instruction::DumpRegistersTill(reg) => {
                self.pc += 2;
                for i in 0..=(reg as u8) {
                    self.memory[self.index + (i as usize)] = self.registers[i as usize];
                }
            }
            Instruction::LoadRegistersTill(reg) => {
                self.pc += 2;
                for i in 0..=(reg as u8) {
                    self.registers[i as usize] = self.memory[self.index + (i as usize)];
                }
            }

            _ => {}
        }
    }



    fn rerender(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0,0,0));
        self.canvas.clear();
        self.canvas.set_draw_color(Color::RGB(255,255,255));
        for y in 0..32 {
            for x in 0..64 {
                if self.pixel_buffer[y][x] {
                    self.canvas.fill_rect(Rect::new((x*10) as i32, (y*10) as i32, 10, 10)).unwrap();
                }
            }
        }
        self.canvas.present();
    }


    fn get_key_press(&mut self) -> u8 {
        // TODO this should be blocking
        return '1' as u8;
    }

    fn clear_screen(&mut self) {
        self.pixel_buffer = vec![vec![false; 64]; 32];
        self.rerender()
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
            println!("beep beep!");
            self.sound_timer -= 1;
        }
    }

    fn sleep() {
        thread::sleep(time::Duration::from_millis(1));
    }

    fn load_rom(&mut self, data: Vec<u8>) {
        self.memory[0x200..(0x200 + data.len())].copy_from_slice(&data);

    }

}

fn main() {

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", 640, 320)
        .position_centered()
        .build()
        .unwrap();
    // window.set_fullscreen(FullscreenType::True).unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0,0,0));
    canvas.clear();
    canvas.present();
    let mut c8 = Chip8::new(canvas);

    let mut data: Vec<u8> = Vec::new();
    // File::open("roms/INVADERS").unwrap().read_to_end(&mut data);
    // File::open("roms/UFO").unwrap().read_to_end(&mut data);
    File::open("roms/TETRIS").unwrap().read_to_end(&mut data).unwrap();
    // File::open("roms/TANK").unwrap().read_to_end(&mut data).unwrap();

    // this should put 5 in top left corner
    // let mut data: Vec<u8> = vec![0x00, 0xE0, 0x61, 0x0d, 0xF1, 0x29, 0x61, 0x3e, 0x62, 0x03, 0xD1, 0x25, 0x00, 0x0F, 0x12, 0x0C];

    c8.load_rom(data);
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        let oc = c8.fetch();
        let inst = c8.decode(oc);
        c8.execute(inst);

        c8.update_delay_timer();
        c8.update_sound_timer();

        // we need to run at about 60hz
        Chip8::sleep();
    }
}
