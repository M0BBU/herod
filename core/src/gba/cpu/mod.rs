mod arm7tdmi;

use crate::gba::bus;

pub struct Cpu {
    processor: arm7tdmi::Processor,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            processor: arm7tdmi::Processor::new(),
        }
    }

    pub fn power() {
        //println!("CPU powering one!");
    }

    pub fn step(&mut self, clocks: u32, bus: &mut bus::Bus) {
        self.processor.step(clocks, bus);
    }
}
