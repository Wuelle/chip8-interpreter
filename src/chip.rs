use anyhow::{anyhow, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample, Stream, StreamConfig,
};
use minifb::{Key, Window};
use rand::Rng;
use std::convert::TryInto;
use std::fs;
use std::io::Read;

fn beep(data: &mut [f32], next: &mut dyn FnMut() -> f32) {
    for sample in data.iter_mut() {
        *sample = Sample::from(&next());
    }
}

fn index_to_key(ix: u8) -> Key {
    match ix {
        0x00 => Key::X,
        0x01 => Key::Key1,
        0x02 => Key::Key2,
        0x03 => Key::Key3,
        0x04 => Key::Q,
        0x05 => Key::W,
        0x06 => Key::E,
        0x07 => Key::A,
        0x08 => Key::S,
        0x09 => Key::D,
        0x0A => Key::Y,
        0x0B => Key::C,
        0x0C => Key::Key4,
        0x0D => Key::R,
        0x0E => Key::F,
        0x0F => Key::V,
        _ => panic!("Invalid Key Index: {}", ix),
    }
}

pub struct Chip {
    memory: [u8; 0xFFF],
    registers: [u8; 16],
    address_register: u16,
    program_counter: u16,
    stack: Vec<u16>,
    pub sound_timer: u8,
    pub delay_timer: u8,
    pub screendata: [[u8; 64]; 32],
    pub stream: Stream,
}

impl Chip {
    pub fn new() -> Self {
        let mut memory = [0; 0xFFF];
        let font = [
            // 0
            [0xF0, 0x90, 0x90, 0x90, 0xF0],
            // 1
            [0x20, 0x60, 0x20, 0x20, 0x70],
            // 2
            [0xF0, 0x10, 0xF0, 0x80, 0xF0],
            // 3
            [0xF0, 0x10, 0xF0, 0x10, 0xF0],
            // 4
            [0x90, 0x90, 0xF0, 0x10, 0x10],
            // 5
            [0xF0, 0x80, 0xF0, 0x10, 0xF0],
            // 6
            [0xF0, 0x80, 0xF0, 0x90, 0xF0],
            // 7
            [0xF0, 0x10, 0x20, 0x40, 0x40],
            // 8
            [0xF0, 0x90, 0xF0, 0x90, 0xF0],
            // 9
            [0xF0, 0x90, 0xF0, 0x10, 0xF0],
            // A
            [0xF0, 0x90, 0xF0, 0x90, 0x90],
            // B
            [0xE0, 0x90, 0xE0, 0x90, 0xE0],
            // C
            [0xF0, 0x80, 0x80, 0x80, 0xF0],
            // D
            [0xE0, 0x90, 0x90, 0x90, 0xE0],
            // E
            [0xF0, 0x80, 0xF0, 0x80, 0xF0],
            // F
            [0xF0, 0x80, 0xF0, 0x80, 0x80],
        ];

        // use the const generic version here
        for (mem, c) in memory.chunks_exact_mut(5).zip(&font) {
            for (ix, elem) in mem.iter_mut().enumerate() {
                *elem = c[ix];
            }
        }

        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("no output device available");

        let config: StreamConfig = device.default_output_config().unwrap().into();

        // Produce a sinusoid of maximum amplitude.
        let sample_rate = 4096.;
        let mut sample_clock = 0.;
        let mut next_value = move || {
            sample_clock = (sample_clock + 1.0) % sample_rate;
            (sample_clock * 15.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
        };

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| beep(data, &mut next_value),
                |_e| eprintln!("error!"),
            )
            .unwrap();

        Self {
            memory: memory,
            registers: [0; 16],
            address_register: 0,
            program_counter: 0x200, // everything up to 0x1FF is for the interpreter
            stack: vec![],
            sound_timer: 1,
            delay_timer: 0,
            screendata: [[0; 64]; 32],
            stream: stream,
        }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<()> {
        // Load game ROM
        let mut file = fs::File::open(path)?;
        file.read(&mut self.memory[0x200..0xFFF])?;
        Ok(())
    }

    pub fn step(&mut self, window: &Window) -> Result<()> {
        // fetch
        let op = ((self.memory[self.program_counter as usize] as u16) << 8)
            | self.memory[self.program_counter as usize + 1] as u16;
        self.program_counter += 2;
        // println!("executing op {:#06x}", op);

        // decode & execute
        match op & 0xF000 {
            0x0000 => {
                match op & 0x000F {
                    0x0000 => {
                        // clear screen
                        self.screendata = [[0; 64]; 32];
                    }
                    0x000E => {
                        // return from subroutine
                        self.program_counter = self
                            .stack
                            .pop()
                            .ok_or(anyhow!("Trying to pop empty Stack"))?;
                    }
                    _ => {
                        panic!("unimplemented OpCode: {:#10x}", op)
                    }
                }
            }
            0x1000 => {
                // jump
                self.program_counter = op & 0x0FFF;
            }
            0x2000 => {
                // call subroutine
                self.stack.push(self.program_counter);
                self.program_counter = op & 0x0FFF;
            }
            0x3000 => {
                // skip the next instruction if VX == NN
                let nn = op & 0x00FF;
                let reg_x = ((op & 0x0F00) >> 8) as usize;
                if nn == self.registers[reg_x].into() {
                    self.program_counter += 2;
                }
            }
            0x4000 => {
                // skip the next instruction if VX == NN
                let nn = op & 0x00FF;
                let reg_x = ((op & 0x0F00) >> 8) as usize;
                if nn != self.registers[reg_x].into() {
                    self.program_counter += 2;
                }
            }
            0x5000 => {
                // skip the next instruction if VX == VY
                let reg_x = ((op & 0x0F00) >> 8) as usize;
                let reg_y = ((op & 0x00F0) >> 4) as usize;
                if self.registers[reg_x] == self.registers[reg_y] {
                    self.program_counter += 2;
                }
            }
            0x6000 => {
                // Set VX to NN
                let reg_x = ((op & 0x0F00) >> 8) as usize;
                self.registers[reg_x] = (op & 0x00FF).try_into()?;
            }
            0x7000 => {
                // Add NN to VX (carry flag is not changed)
                let reg_x = ((op & 0x0F00) >> 8) as usize;
                self.registers[reg_x] =
                    self.registers[reg_x].wrapping_add((op & 0x00FF).try_into()?);
            }
            0x8000 => {
                match op & 0x000F {
                    0x0000 => {
                        // Set VX to VY
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        let reg_y = ((op & 0x00F0) >> 4) as usize;
                        self.registers[reg_x] = self.registers[reg_y];
                    }
                    0x0001 => {
                        // Set VX to VX OR VY
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        let reg_y = ((op & 0x00F0) >> 4) as usize;
                        self.registers[reg_x] |= self.registers[reg_y];
                    }
                    0x0002 => {
                        // Set VX to VX AND VY
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        let reg_y = ((op & 0x00F0) >> 4) as usize;
                        self.registers[reg_x] &= self.registers[reg_y];
                    }
                    0x0003 => {
                        // Set VX to VX XOR VY
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        let reg_y = ((op & 0x00F0) >> 4) as usize;
                        self.registers[reg_x] ^= self.registers[reg_y];
                    }
                    0x0004 => {
                        // Add VY to VX where VF represents the carry
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        let reg_y = ((op & 0x00F0) >> 4) as usize;
                        let res = self.registers[reg_x].overflowing_add(self.registers[reg_y]);
                        self.registers[15] = if res.1 { 1 } else { 0 };
                        self.registers[reg_x] = res.0;
                    }
                    0x0005 => {
                        // Subtract VY from VX, VF is set to 0 if there was a borrow and 1 if not
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        let reg_y = ((op & 0x00F0) >> 4) as usize;
                        let res = self.registers[reg_x].overflowing_sub(self.registers[reg_y]);
                        self.registers[15] = if res.1 { 0 } else { 1 };
                        self.registers[reg_x] = res.0;
                    }
                    0x0006 => {
                        // store the least significant bit of VX in VF and shift VX to the right by 1
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        self.registers[15] = self.registers[reg_x] & 0x0001;
                        self.registers[reg_x] >>= 1;
                    }
                    0x0007 => {
                        // Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there is not.
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        let reg_y = ((op & 0x00F0) >> 4) as usize;
                        let res = self.registers[reg_y].overflowing_sub(self.registers[reg_x]);
                        self.registers[15] = if res.1 { 0 } else { 1 };
                        self.registers[reg_x] >>= 1;
                    }
                    0x000E => {
                        // Store the most significant bit of VX in VF and shift VX to the left by 1
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        self.registers[15] = self.registers[reg_x] & 0x10;
                        self.registers[reg_x] >>= 1;
                    }
                    _ => {
                        panic!("unimplemented OpCode: {:#10x}", op)
                    }
                }
            }
            0x9000 => {
                // Skips the next instruction if VX does not equal VY
                let reg_x = ((op & 0x0F00) >> 8) as usize;
                let reg_y = ((op & 0x00F0) >> 4) as usize;
                if self.registers[reg_x] != self.registers[reg_y] {
                    self.program_counter += 2;
                }
            }
            0xA000 => {
                // Set address register to adress NNN
                self.address_register = op & 0x0FFF;
            }
            0xB000 => {
                // Jump to the adress NNN + V0
                self.program_counter = self.registers[0] as u16 + op & 0x0FFF
            }
            0xC000 => {
                // Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
                let reg_x = ((op & 0x0F00) >> 8) as usize;
                let nn: u8 = (op & 0x00FF).try_into()?;
                let r: u8 = rand::thread_rng().gen();
                self.registers[reg_x] = nn & r;
            }
            0xD000 => {
                // Draw a sprite at VX, VY with data starting at the address register
                let reg_x = ((op & 0x0F00) >> 8) as usize;
                let reg_y = ((op & 0x00F0) >> 4) as usize;
                let height = (op & 0x000F) as usize;

                let x_start = self.registers[reg_x] as usize;
                let y_start = self.registers[reg_y] as usize;
                self.registers[15] = 0;

                for line in 0..height {
                    let y = y_start + line;
                    let data = self.memory[self.address_register as usize + line];

                    for bit_ix in 0..8 {
                        let x = x_start + bit_ix;
                        let mask = 1 << (7 - bit_ix);
                        // println!("mask: {:#08b}", mask);
                        // println!("matc: {:#08b}", data & mask);

                        // only flip the pixel if the corresponding bit is turned on
                        if data & mask != 0 {
                            // check for collisions
                            if self.screendata[y][x] == 1 {
                                self.registers[15] = 1;
                            }
                            // flip the pixels activation
                            self.screendata[y][x] ^= 1;
                        }
                    }
                }
            }
            0xE000 => {
                match op & 0x00FF {
                    0x009E => {
                        // Skips the next instruction if the key stored in VX is pressed
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        let k = index_to_key(self.registers[reg_x]);
                        if window.is_key_down(k) {
                            self.program_counter += 2;
                        }
                    }
                    0x00A1 => {
                        // Skips the next instruction if the key stored in VX is not pressed
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        let k = index_to_key(self.registers[reg_x]);
                        if !window.is_key_down(k) {
                            self.program_counter += 2;
                        }
                    }
                    _ => {
                        panic!("unimplemented OpCode: {:#10x}", op)
                    }
                }
            }
            0xF000 => {
                match op & 0x00FF {
                    0x0007 => {
                        // Set the VX to the delay timer
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        self.registers[reg_x] = self.delay_timer;
                    }
                    0x000A => {
                        unimplemented!();
                        // A key press is awaited, then stored in VX (blocking)
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        let key_pressed = false;
                        while !key_pressed {}
                    }
                    0x0015 => {
                        // Set the delay timer to VX
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        self.delay_timer = self.registers[reg_x];
                    }
                    0x0018 => {
                        // Set the sound timer to VX
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        self.sound_timer = self.registers[reg_x];
                    }
                    0x001E => {
                        // Add VX to I. VF is not affected
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        self.address_register += self.registers[reg_x] as u16;
                    }
                    0x0029 => {
                        // Sets I to the location of the sprite stored in VX
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        let sprite = self.registers[reg_x] as u16;
                        self.address_register = 0x000 + 0x005 * sprite
                    }
                    0x0033 => {
                        // Stores the binary decimal representation of VX in I
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        let val = self.registers[reg_x];

                        let hundreds = val / 100;
                        let tens = (val * 10) % 10;
                        let ones = val % 10;
                        let base = self.address_register as usize;

                        self.memory[base] = hundreds;
                        self.memory[base + 1] = tens;
                        self.memory[base + 2] = ones;
                    }
                    0x0055 => {
                        // Stores V0 to VX (including VX) at I
                        let reg_x = ((op & 0x0F00) >> 8) as usize;

                        for ix in 0..reg_x {
                            self.memory[self.address_register as usize + ix] = self.registers[ix];
                        }
                    }
                    0x0065 => {
                        // Read V0 to VX (including VX) from I
                        let reg_x = ((op & 0x0F00) >> 8) as usize;

                        for ix in 0..reg_x {
                            self.registers[ix] = self.memory[self.address_register as usize + ix]
                        }
                    }
                    _ => panic!("unimplemented OpCode: {:#10x}", op),
                }
            }

            _ => {
                panic!("unimplemented OpCode: {:#10x}", op)
            }
        }
        Ok(())
    }
}
