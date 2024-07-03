pub struct Ppu {
    vram: Vec<u8>,
    palette: Vec<u32>,
    pram: Vec<u8>,
    output: Vec<u32>,
    io_regs: Io,
}

struct Io {
    disp_ctrl: u16,
    disp_stat: u16,
    v_count: u16,
}

impl Ppu {
    pub fn new() -> Ppu {
        // VRAM is 96kb
        // PRAM is 1kb (512 bytes for BG and 512 for OBJ)
        // Dunno if vram is zero initialized
        // Dimensions of GBA screen is 240 x 160
        Ppu {
            vram: vec![0; 96 * 1024],
            palette: vec![0; 512],
            pram: vec![0; 1 * 1024],
            output: vec![0x0; 240 * 160],
            io_regs: Io::new(),
        }
    }

    pub fn read_vram(&self, address: u32) -> u8 {
        // I believe that this ensures that we mirror the memory?
        // It basically wraps around the memory or something
        let mask = if (address >> 17) & 0x01 == 1 {
            0x01_7F_FF
        } else {
            0x00_FF_FF
        };

        let index = (address & mask) as usize;
        self.vram[index]
    }

    pub fn read_pram(&self, address: u32) -> u8 {
        // I believe that this ensures that we mirror the memory?
        // It basically wraps around the memory or something
        let mask = if (address >> 17) & 0x01 == 1 {
            0x01_7F_FF
        } else {
            0x00_FF_FF
        };

        let index = (address & mask) as usize;
        self.vram[index]
    }

    pub fn read_io(&self, address: u32) -> u8 {
        match address {
            0x0400_0000 => self.io_regs.disp_ctrl as u8,
            0x0400_0001 => (self.io_regs.disp_ctrl >> 8) as u8,
            0x0400_0004 => self.io_regs.disp_stat as u8,
            0x0400_0005 => (self.io_regs.disp_stat >> 8) as u8,
            0x0400_0006 => (self.io_regs.v_count) as u8,
            _ => unimplemented!("Invalid address {:#2X}!", address),
        }
    }

    pub fn write_vram(&mut self, address: u32, value: u8) {
        // I believe that this ensures that we mirror the memory?
        // It basically wraps around the memory or something
        let mask = if (address >> 17) & 0x01 == 1 {
            0x01_7F_FF
        } else {
            0x01_FF_FF
        };
        let index = (address & mask) as usize;
        self.vram[index] = value;
    }

    pub fn write_pram(&mut self, address: u32, value: u8) {
        // I believe that this ensures that we mirror the memory?
        // It basically wraps around the memory or something
        let index = (address & 0x03_FF) as usize;
        self.pram[index] = value;

        // Update the palette from here?
        let palette_idx = (address & 0x03_FE) as usize;
        let pixel = u32::from(self.pram[palette_idx]) | u32::from(self.pram[palette_idx + 1]) << 8;

        let r: u32 = (pixel & 0x1F) << 3;
        let g: u32 = ((pixel >> 5) & 0x1F) << 3;
        let b: u32 = ((pixel >> 10) & 0x1F) << 3;

        // Set the alpha first
        let mut rgba: u32 = 0xFF_00_00_00;

        rgba |= r << 16;
        rgba |= g << 8;
        rgba |= b;

        self.palette[palette_idx] = rgba;
    }

    pub fn write_io(&mut self, address: u32, value: u8) {
        match address {
            0x0400_0000 => self.io_regs.disp_ctrl |= u16::from(value),
            0x0400_0001 => self.io_regs.disp_ctrl |= u16::from(value) << 8,
            0x0400_0004 => self.io_regs.disp_stat |= u16::from(value),
            0x0400_0005 => self.io_regs.disp_stat |= u16::from(value) << 8,
            _ => log::error!("Invalid address {:#2X}!", address),
            //_ => unimplemented!("Invalid address {:#2X}!", address),
        }
    }

    pub fn start_hblank(&mut self) {
        // Do we always set the HBLANK flag?
        self.io_regs.disp_stat |= 0b10;
    }

    pub fn end_hblank(&mut self) {
        // Do we always set the HBLANK flag?
        self.io_regs.disp_stat &= !0b10;
    }

    // Certaintly not accurate at all, but this should do for now.
    // Main goal is to get the CPU up and running and some basic games
    // maybe.
    // Do I need to update the framebuffer per line?
    pub fn render_line(&mut self) {
        // Bit 0 = VBLANK, Bit 1 = HBLANK
        if self.io_regs.v_count == 268 {
            self.io_regs.v_count = 0;
            // Reset VBLANK?
            self.io_regs.disp_stat &= !0b01;
            return;
        }
        if self.io_regs.v_count >= 160 {
            // This is when we set the VBLANK?
            self.io_regs.disp_stat |= 0b01;
            // println!("DISP STAT STAT is {}", self.io_regs.disp_stat);
            self.io_regs.v_count += 1;
            return;
        }

        let mode = self.io_regs.disp_ctrl & 0b111;
        match mode {
            3 => {
                let buffer_addr = self.io_regs.v_count * 240;
                let mut vram_addr: u32 = u32::from(self.io_regs.v_count) * 480;
                for i in 0..240 {
                    let pixel: u32 = u32::from(self.vram[vram_addr as usize])
                        | (u32::from(self.vram[(vram_addr | 1) as usize]) << 8);

                    let r: u32 = (pixel & 0x1F) << 3;
                    let g: u32 = ((pixel >> 5) & 0x1F) << 3;
                    let b: u32 = ((pixel >> 10) & 0x1F) << 3;

                    // Set the alpha first
                    let mut rgba: u32 = 0xFF_00_00_00;

                    // Why do I need this?
                    // rgba |= (r | (r >> 5)) << 16;
                    // rgba |= (g | (g >> 5)) << 8;
                    // rgba |= (b | (b >> 5)) << 0;

                    rgba |= r << 16;
                    rgba |= g << 8;
                    rgba |= b;

                    self.output[(i + buffer_addr) as usize] = rgba;
                    vram_addr += 2;
                }
            }
            4 => {
                let buffer_addr = self.io_regs.v_count * 240;
                let window0 = (self.io_regs.disp_ctrl >> 13) & 0x01 == 1;
                let mut vram_addr = if window0 {
                    // Frame 0
                    u32::from(self.io_regs.v_count) * 240
                } else {
                    // Frame 1
                    (u32::from(self.io_regs.v_count) * 240) + 0xA0_00
                };

                for i in 0..240 {
                    let idx = self.vram[vram_addr as usize] as usize;
                    self.output[(i + buffer_addr) as usize] = self.palette[idx];
                    vram_addr += 1;
                }
            }
            _ => panic!("Video mode {} not implemented yet!", mode),
        }
        self.io_regs.v_count += 1;
    }

    pub fn render_screen(&self) -> &Vec<u32> {
        &self.output
    }
}

impl Io {
    fn new() -> Io {
        Io {
            disp_ctrl: 0x0,
            disp_stat: 0x0,
            v_count: 0x0,
        }
    }
}
