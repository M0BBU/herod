use herod_gba_core::gba;

use minifb::{Key, Window, WindowOptions, Scale};
use simple_logger::SimpleLogger;
use log::LevelFilter;

const WIDTH: usize = 240;
const HEIGHT: usize = 160;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .init()
        .unwrap();
    let mut buffer: &Vec<u32>;

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {

            resize: true,
            scale: Scale::X4,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
       panic!("{}", e);
    });

    let mut test_gba = gba::HerodGBA::new();
    test_gba.power();
    test_gba.load_cartridge_from_args();

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer = test_gba.render_frame();

       // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
       window
           .update_with_buffer(&buffer, WIDTH, HEIGHT)
           .unwrap();
    }
}
