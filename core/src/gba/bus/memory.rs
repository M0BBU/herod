pub struct Memory {
    wram_board: Vec<u8>,
    wram_chip: Vec<u8>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            // 256 KB for on board Work RAM but I believe it is slower (2 wait)?
            // at 0x2000000
            // 32 KB for on chip Work RAM which is faster
            // at 0x3000000
            wram_board: vec![0; 256 * 1024],
            wram_chip: vec![0; 32 * 1024],
        }
    }

    pub fn read_wram(&self, address: u32) -> u8 {
        match address >> 24 {
            0x2 => self.wram_board[(address & 0x03_FF_FF) as usize],
            0x3 => self.wram_chip[(address & 0x00_7F_FF) as usize],
            _ => panic!(
                "Trying to read WRAM at {:#2X}, should not have happened!",
                address
            ),
        }
    }

    pub fn write_wram(&mut self, address: u32, value: u8) {
        match address >> 24 {
            0x2 => self.wram_board[(address & 0x03_FF_FF) as usize] = value,
            0x3 => self.wram_chip[(address & 0x00_7F_FF) as usize] = value,
            _ => panic!(
                "Trying to write to WRAM at {:#2X}, should not have happened!",
                address
            ),
        }
    }
}
