#![allow(dead_code)]
use herod_gba_core::gba;

use log::LevelFilter;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();

    let mut test_gba = gba::HerodGBA::new();
    test_gba.power();
    test_gba.load_cartridge_from_args();

    // benchmark(&mut test_gba);
    loop {
        test_gba.render_frame();
        //println!("Rendered {} frames", i);
    }
}

fn benchmark(gba: &mut gba::HerodGBA) {
    use std::time::Instant;
    let now = Instant::now();
    {
        gba.render_frame();
    }
    let elapsed = now.elapsed();
    println!("Elapsed : {:.2?}", elapsed);
}
