mod bus;
mod cartridge;
mod cpu;
mod ppu;

const LINES_TOTAL: u32 = 228;
const LINES_VISIBLE: u32 = 160;

const CYCLES_TOTAL_PER_LINE: u32 = 1232;
const CYCLES_TOTAL_HBLANK0: u32 = 1006;
const CYCLES_TOTAL_HBLANK1: u32 = CYCLES_TOTAL_PER_LINE - CYCLES_TOTAL_HBLANK0;

pub struct HerodGBA {
    cpu: cpu::Cpu,
    bus: bus::Bus,
}

impl HerodGBA {
    pub fn new() -> HerodGBA {
        let m = bus::memory::Memory::new();
        let c = cartridge::Cartridge::new();
        let p = ppu::Ppu::new();

        HerodGBA {
            cpu: cpu::Cpu::new(),
            bus: bus::Bus::new(m, c, p),
        }
    }

    pub fn power(&mut self) {
        //println!("herodGBA running!");
    }

    pub fn load_cartridge_from_args(&mut self) {
        let file_name = std::env::args().nth(1).expect("Please specify a ROM!");
        //println!("Running rom {file_name}");

        self.bus.cartridge.load(&file_name);
    }

    pub fn render_frame(&mut self) -> &Vec<u32> {
        let mut cycles = 0;
        //let mut i = 1;
        while cycles < 280896 {
            // Do something with HBLANK0 here?
            self.cpu.step(CYCLES_TOTAL_HBLANK0, &mut self.bus);

            self.bus.ppu.start_hblank();
            // Do something with HBLANK1 here?
            self.bus.ppu.render_line();
            self.bus.ppu.end_hblank();

            self.cpu.step(CYCLES_TOTAL_HBLANK1, &mut self.bus);

            cycles += CYCLES_TOTAL_PER_LINE;
            // println!("Rendered {} lines", i);
            //i += 1;
        }
        self.bus.ppu.render_screen()
    }
}
