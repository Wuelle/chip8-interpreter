mod chip;

use anyhow::Result;
use chip::Chip;
use minifb::{Key, Window, WindowOptions, Scale};
use std::io::BufReader;

fn main() -> Result<()> {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    let file = std::fs::File::open("beep.wav").unwrap();
    let beep1 = stream_handle.play_once(BufReader::new(file)).unwrap();
    beep1.set_volume(0.2);
    std::thread::sleep(std::time::Duration::from_millis(1500));

    let mut chip = Chip::new();
    chip.load_rom("rom/PONG")?;

    let mut buffer = [0; 64 * 32];

    let mut options = WindowOptions::default();
    options.scale = Scale::X16;
    let mut window = Window::new("Chip-8 Emulator - ESC to exit", 64, 32, options)?;

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        chip.step(&window);

        // copy screendata into the buffer
        for x in 0..64 {
            for y in 0..32 {
                let g = chip.screendata[y][x] as u32;
                buffer[y * 64 + x] = (g << 16) | (g << 8) | g;
            }
        }

        window.update_with_buffer(&buffer, 64, 32)?;
    }

    Ok(())
}
