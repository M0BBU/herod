pub struct Cartridge {
    rom: Rom,
}

struct Rom {
    data: Vec<u8>,
}

impl Cartridge {
    pub fn new() -> Cartridge {
        Cartridge { rom: Rom::new() }
    }

    pub fn load(&mut self, file_name: &str) {
        self.rom.data = std::fs::read(file_name).expect("Could not read ROM!");
    }

    pub fn read_rom(&self, address: u32) -> u8 {
        let index = (address & 0x1_FF_FF_FF) as usize;
        // Do I need to check for overflow here?
        if index < self.rom.data.len() {
            self.rom.data[index]
        } else {
            0x0
        }
    }
}

impl Rom {
    fn new() -> Rom {
        // I don't believe the ROM is zero initialized?
        Rom {
            data: vec![0; 32 * 1024 * 1024],
        }
    }
}
