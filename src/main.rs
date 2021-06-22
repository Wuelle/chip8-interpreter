mod chip;

use anyhow::Result;
use chip::Chip;

fn main() -> Result<()> {
    let mut chip = Chip::new();
    chip.load_rom("rom/tetris.rom")?;

    // run some steps
    for _ in 0..1000 {
        chip.step();
    }

    Ok(())
}
