use anyhow::Result;
use opcodes::OpCode;
use std::io::Read;
use std::fs;

pub struct Chip {
    memory: [u8; 0xFFF],
    registers: [u8; 16],
    address_register: u16,
    program_counter: u16,
    stack: Vec<u16>,
    screendata: [u8; 64 * 32],
}

impl Chip {
    pub fn new() -> Self {
        Self {
            memory: [0; 0xFFF],
            registers: [0; 16],
            address_register: 0,
            program_counter: 0x200, // everything up to 0x1FF is for the interpreter
            stack: vec![],
            screendata: [0; 64 * 32],
        }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<()> {
        // Load game ROM
        let mut file = fs::File::open("rom/tetris.rom")?;
        let mut rom = [0; 0xFFF - 0x200];
        file.read(&mut rom)?;
        Ok(())
    }

    pub fn next_opcode(&mut self) -> OpCode {
        let op = ((self.memory[self.program_counter as usize] as u16) << 8) | self.memory[self.program_counter as usize] as u16;
        self.program_counter += 2;

        match op && 0xF000 {
            0x1000 => {
                // jump
                self.program_counter = op && 0x0FFF;
            }, 
            0x0000 => {
                match op && 0x000F {
                    0x0000 => {
                        return OpCode::_00E0 // clear screen
                    },
                    0x000E => {
                        return OpCode::_00EE // return from subroutine
                    },
                    _ => { panic!("unimplemented OpCode: {}", op},

                }
            },
            _ => { panic!("unimplemented OpCode: {}", op},
        }

    }
}
