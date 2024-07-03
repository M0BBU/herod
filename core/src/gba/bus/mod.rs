use crate::gba::cartridge;
use crate::gba::ppu;

pub mod memory;

// I don't think I need this anymore.
//#[derive(PartialEq, Eq)]
//pub enum Mode {
//    Word,
//    Half,
//    Byte,
//}

pub struct Bus {
    pub mem: memory::Memory,
    pub cartridge: cartridge::Cartridge,
    pub ppu: ppu::Ppu,
}

impl Bus {
    pub fn new(mem: memory::Memory, cartridge: cartridge::Cartridge, ppu: ppu::Ppu) -> Bus {
        Bus {
            mem,
            cartridge,
            ppu,
        }
    }

    pub fn read_word(&mut self, address: u32) -> u32 {
        // Memory reads need to be aligned as per
        // https://problemkaputt.de/gbatek-arm-cpu-memory-alignments.htm
        // Reads from forcibly aligned addresses need to be rotated
        // by the amount it was mis-aligned * 8, hence the shift val.
        let aligned_addr = address & !3;
        let shift = address & 3;
        let value = u32::from(self.read_byte(aligned_addr))
            | u32::from(self.read_byte(aligned_addr | 1)) << 8
            | u32::from(self.read_byte(aligned_addr | 2)) << 16
            | u32::from(self.read_byte(aligned_addr | 3)) << 24;

        value.rotate_right(shift << 3)
    }

    pub fn read_half(&mut self, address: u32) -> u32 {
        let aligned_addr = address & !1;
        let shift = address & 1;
        log::debug!("Aligned address is {:#2X}", aligned_addr);
        let value = u32::from(self.read_byte(aligned_addr))
            | u32::from(self.read_byte(aligned_addr | 1)) << 8;

        // TODO: Might need to do more stuff here

        value.rotate_right(shift << 3)
    }

    pub fn read_byte(&mut self, address: u32) -> u8 {
        match address >> 24 {
            0x08..=0x0B => self.cartridge.read_rom(address),
            0x06 => self.ppu.read_vram(address),
            // This isn't necessarily right because some io registers belong to
            // sound channels I believe. Need to check for that?
            0x04 => self.ppu.read_io(address),
            0x02..=0x03 => self.mem.read_wram(address),
            _ => unimplemented!("Invalid address {:#2X}!", address),
        }
    }

    pub fn write_word(&mut self, address: u32, value: u32) {
        let aligned_addr = address & !3;

        self.write_byte(aligned_addr, value as u8);
        self.write_byte(aligned_addr | 1, (value >> 8) as u8);
        self.write_byte(aligned_addr | 2, (value >> 16) as u8);
        self.write_byte(aligned_addr | 3, (value >> 24) as u8);
    }

    pub fn write_half(&mut self, address: u32, value: u32) {
        let aligned_addr = address & !1;

        self.write_byte(aligned_addr, value as u8);
        self.write_byte(aligned_addr | 1, (value >> 8) as u8);
    }

    pub fn write_byte(&mut self, address: u32, value: u8) {
        match address >> 24 {
            0x06 => self.ppu.write_vram(address, value),
            0x05 => self.ppu.write_pram(address, value),
            0x04 => self.ppu.write_io(address, value),
            0x02..=0x03 => self.mem.write_wram(address, value),
            _ => unimplemented!("Invalid address {:#2X}!", address),
        }
    }
}
