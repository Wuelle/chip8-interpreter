use anyhow::Result;
use rand::Rng;
use std::convert::TryInto;
use std::fs;
use std::io::Read;
use minifb::{Key, Window};

fn index_to_key(ix: u8) -> Key {
    match ix {
        0x00 => Key::Key0,
        0x01 => Key::Key1,
        0x02 => Key::Key2,
        0x03 => Key::Key3,
        0x04 => Key::Key4,
        0x05 => Key::Key5,
        0x06 => Key::Key6,
        0x07 => Key::Key7,
        0x08 => Key::Key8,
        0x09 => Key::Key9,
        0x0A => Key::A,
        0x0B => Key::B,
        0x0C => Key::C,
        0x0D => Key::D,
        0x0E => Key::E,
        0x0F => Key::F,
        _ => panic!("Invalid Key Index: {}", ix),
    }
}

pub struct Chip {
    memory: [u8; 0xFFF],
    registers: [u8; 16],
    address_register: u16,
    program_counter: u16,
    stack: Vec<u16>,
    pub screendata: [[u8; 64]; 32],
}

impl Chip {
    pub fn new() -> Self {
        Self {
            memory: [0; 0xFFF],
            registers: [0; 16],
            address_register: 0,
            program_counter: 0x200, // everything up to 0x1FF is for the interpreter
            stack: vec![],
            screendata: [[0; 64]; 32],
        }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<()> {
        // Load game ROM
        let mut file = fs::File::open(path)?;
        file.read(&mut self.memory[0x200..0xFFF])?;
        Ok(())
    }

    pub fn step(&mut self, window: &Window) {
        // fetch
        let op = ((self.memory[self.program_counter as usize] as u16) << 8)
            | self.memory[self.program_counter as usize + 1] as u16;
        self.program_counter += 2;
        println!("executing op {:#10x}", op);

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
                        self.program_counter = self.stack.pop().unwrap();
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
                self.registers[reg_x] = (op & 0x00FF).try_into().unwrap();
            }
            0x7000 => {
                // Add NN to VX (carry flag is not changed)
                let reg_x = ((op & 0x0F00) >> 8) as usize;
                self.registers[reg_x] =
                    self.registers[reg_x].wrapping_add((op & 0x00FF).try_into().unwrap());
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
                self.address_register = self.memory[(op & 0x0FFF) as usize] as u16;
            }
            0xB000 => {
                // Jump to the adress NNN + V0
                self.program_counter = self.registers[0] as u16 + op & 0x0FFF
            }
            0xC000 => {
                // Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
                let reg_x = ((op & 0x0F00) >> 8) as usize;
                let nn: u8 = (op & 0x00FF).try_into().unwrap();
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
                    },
                }
            }
            0xF000 => {
                match op & 0x00FF {
                    0x0007 => {
                        // Set the VX to the delay timer
                        unimplemented!();
                    }
                    0x000A => {
                        // A key press is awaited, then stored in VX (blocking)
                        unimplemented!();
                    }
                    0x0015 => {
                        // Set the delay timer to VX
                        unimplemented!();
                    }
                    0x0018 => {
                        // Set the sound timer to VX
                        unimplemented!();
                    }
                    0x001E => {
                        // Add VX to I. VF is not affected
                        unimplemented!();
                    }
                    0x0029 => {
                        // Sets I to the location of the sprite stored in VX
                        unimplemented!();
                    }
                    0x0033 => {
                        // Stores the binary decimal representation of VX in I
                        let reg_x = ((op & 0x0F00) >> 8) as usize;
                        let val = self.registers[reg_x];

                        let hundreds = val / 100;
                        let tens = (val * 10) % 10;
                        let ones = val % 10;
                        let base = self.address_register as usize;
                        
                        self.registers[base] = hundreds;
                        self.registers[base + 1] = tens;
                        self.registers[base + 2] = ones;
                    }
                    0x0055 => {
                        // Stores V0 to VX (including VX) at I
                        unimplemented!();
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
    }
}
