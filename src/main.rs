mod opcodes;
mod chip;

use anyhow::Result;
use opcodes::OpCode;
use chip::Chip;

fn main() -> Result<()> {
    let mut chip = Chip::new();
    chip.load_rom("rom/tetris.rom")?;
    println!("opcode: {}", chip.next_opcode());

    Ok(())
}
