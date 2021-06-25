mod chip;

use anyhow::Result;
use chip::Chip;
use cpal::traits::StreamTrait;
use minifb::{Key, Scale, Window, WindowOptions};
use std::{
    thread,
    time::Duration,
};

fn main() -> Result<()> {
    let mut chip = Chip::new();
    chip.load_rom("rom/PONG")?;

    let mut buffer = [0; 64 * 32];

    let mut options = WindowOptions::default();
    options.scale = Scale::X16;
    let mut window = Window::new("Chip-8 Interpreter - ESC to exit", 64, 32, options)?;

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Clock speed is ~540Hz, so 9 steps per draw/timer update
        for _ in 0..9 {
            chip.step(&window)?;
        }

        // Decrement and handle timers
        if chip.sound_timer != 0 {
            chip.stream.play()?;

            chip.sound_timer -= 1;

            if chip.sound_timer == 0 {
                chip.stream.pause()?;
            }
        }

        if chip.delay_timer != 0 {
            chip.delay_timer -= 1;
        }

        // draw buffer to screen
        for x in 0..64 {
            for y in 0..32 {
                let g = chip.screendata[y][x] as u32 * 255;
                buffer[y * 64 + x] = (g << 16) | (g << 8) | g;
            }
        }

        window.update_with_buffer(&buffer, 64, 32)?;
        thread::sleep(Duration::from_millis(1000 / 60));
    }

    Ok(())
}
