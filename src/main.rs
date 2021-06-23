mod chip;

use anyhow::Result;
use chip::Chip;
use minifb::{Key, Scale, Window, WindowOptions};

fn main() -> Result<()> {
    let mut chip = Chip::new();
    chip.load_rom("rom/PONG")?;

    let mut buffer = [0; 64 * 32];

    let mut options = WindowOptions::default();
    options.scale = Scale::X16;
    let mut window = Window::new("Chip-8 Interpreter - ESC to exit", 64, 32, options)?;

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    let mut count = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        chip.step(&window)?;
        // if count < 6 {
        //     chip.step(&window)?;
        // }
        // count += 1;

        // copy screendata into the buffer
        for x in 0..64 {
            for y in 0..32 {
                let g = chip.screendata[y][x] as u32 * 255;
                buffer[y * 64 + x] = (g << 16) | (g << 8) | g;
            }
        }

        window.update_with_buffer(&buffer, 64, 32)?;
    }
    // println!("{:#?}", chip.screendata);

    Ok(())
}
