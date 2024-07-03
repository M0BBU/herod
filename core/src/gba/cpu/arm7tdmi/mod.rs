use crate::gba::bus;

mod instructions;

use instructions::*;

pub struct Processor {
    regs: Registers,
    pipe: [u32; 2],
    exec_arm: [ArmInstruction; 4096],
}

#[derive(Default)]
struct Registers {
    r: [u32; 13], // General registers

    r13_sp: u32,
    r13_fiq: u32,
    r13_svc: u32,
    r13_abt: u32,
    r13_und: u32,

    r14: u32,
    r14_fiq: u32,
    r14_svc: u32,
    r14_abt: u32,
    r14_und: u32,

    r15_pc: u32,

    cpsr: u32,

    spsr_fiq: u32,
    spsr_svc: u32,
    spsr_abt: u32,
    spsr_irq: u32,
    spsr_und: u32,
}

pub enum PSRFlags {
    Negative,
    Zero,
    Carry,
    Overflow,
    Saturation,
    AbortOff,
    IRQOff,
    FIQOff,
    Thumb
}

impl Processor {
    pub fn new() -> Processor {
        const EXEC_ARM: [ArmInstruction; 4096] = Processor::gen_arm_table();
        Processor {
            regs: Registers {
                r15_pc: 0x08_00_00_00,
                cpsr: 0x00,
                ..Registers::default()
            },
            pipe: [0xF0_00_00_00; 2],
            exec_arm: EXEC_ARM,
        }
    }

    pub fn step(&mut self, clocks: u32, bus: &mut bus::Bus) {
        let mut c = 0;
        while c < clocks {
            let instr = self.pipe[0];
            self.pipe[0] = self.pipe[1];
            self.pipe[1] = bus.read_word(self.regs.r15_pc);

            println!("Running addr {:#2X}", self.regs.r15_pc - 8);
            //println!("Regs {:#2X?}, {:#2X}", self.regs.r, self.regs.r13_sp);
            println!("INSTR IS {:#2X}", instr);
            //println!("CLOCKS {}", c);

            if self.regs.check_cond((instr >> 28) & 0x0F) {
                let hash = ((instr & 0x0F_F0_00_00) >> 16) | ((instr & 0x00_00_00_F0) >> 4);

                let ArmInstruction { name: _, handler } = self.exec_arm[hash as usize];
                handler(self, bus, instr);
            }
            self.regs.r15_pc += 4;
            c += 1;
        }
    }

    pub fn reload_arm_pipeline(&mut self, bus: &mut bus::Bus) {
        // println!("PC is {:#2X}", self.regs.r15_pc);
        self.pipe[0] = bus.read_word(self.regs.r15_pc);
        self.pipe[1] = bus.read_word(self.regs.r15_pc + 4);
        self.regs.r15_pc += 4;
    }

    const fn gen_arm_table() -> [ArmInstruction; 4096] {
        let mut arm_table = [ARM_UNKNOWN; 4096];
        let mut i = 0;
        while i < 4096 {
            let instruction = ((i & 0x0F_F0) << 16) | ((i & 0x0F) << 4);
            arm_table[i as usize] = Processor::arm_decode(instruction);
            i += 1;
        }
        arm_table
    }

    // The decoding logic closely models NanoBoyAdvance's logic
    // See here: https://github.com/nba-emu/NanoBoyAdvance/blob/master/src/nba/src/arm/tablegen/gen_arm.hpp
    const fn arm_decode(instr: u32) -> ArmInstruction {
        // Mask out the condition bits 28 - 31 as we will deal with them later
        let opcode = instr & 0x0F_FF_FF_FF;
        match opcode >> 26 {
            0b00 => {
                if (opcode >> 25) & 0x1 == 1 {
                    let opcode = (instr >> 21) & 0xF;
                    // I might need to check this better?
                    if opcode == 0b1001 || opcode == 1010 {
                        // Should be PSR
                        ARM_STATUS_TRANSFER
                    } else {
                        ARM_DATA_PROC
                    }
                } else if (opcode & 0x0F_F0_00_F0) == 0x01_20_00_10 {
                    // Should be BX -> Branch and Exchange
                    ARM_BRANCH_AND_EXCHANGE
                } else if (opcode & 0x01_00_00_F0) == 0x00_00_00_90 {
                    // We are in MULTIPLY :)
                    if (opcode >> 23) & 0x1 == 1 {
                        // MULTIPLY LONG
                        ARM_MULTIPLY_LONG
                    } else {
                        // MULTIPLY
                        ARM_MULTIPLY
                    }
                } else if (opcode & 0x01_00_00_F0) == 0x01_00_00_90 {
                    // Single Data Transfer which is TransSwp12 on GBATek?
                    ARM_SINGLE_DATA_TRANSFER
                } else if (opcode & 0xF0) == 0xB0 || (opcode & 0xD0) == 0xD0 {
                    // Halfword Data Transfer, register + immediate offset
                    // Signed Data Transfer as well?
                    // In this case, we check for bits 4 - 7 which are 1SH1 per
                    // the encoding. The S and H bit cannot both be 0, so we can
                    // either have 1101 -> D or 1011 -> B
                    // This is the TransReg10 and TransImm10 on GBATek?
                    ARM_HALF_SIGNED_TRANSFER
                } else {
                    let opcode = (instr >> 21) & 0xF;
                    let set_flags = (instr >> 20) & 0b1 == 1;
                    if !set_flags && opcode >= 0b1000 && opcode <= 0b1011 {
                        // PSR Transfer but with register?
                        ARM_STATUS_TRANSFER
                    } else {
                        // Data Processor, first two entries on GBATek
                        ARM_DATA_PROC
                    }
                }
            }
            0b01 => {
                // Might need to check for an undefined instruction here?
                // For now, we'll just return the Single Data Transfer
                ARM_SINGLE_DATA_TRANSFER
            }
            0b10 => {
                if ((opcode >> 25) & 0x01) == 1 {
                    ARM_BRANCH_AND_LINK
                } else {
                    ARM_BLOCK_DATA_TRANSFER
                }
            }
            0b11 => ARM_UNKNOWN, // Should do something about interrupts and coprocessor?
            _ => panic!("Should not have happened!"),
        }
    }
}

impl Registers {
    pub fn get_reg(&self, reg: u32) -> u32 {
        match reg {
            0..=12 => self.r[reg as usize],
            13 => self.r13_sp,
            14 => self.r14,
            15 => self.r15_pc,
            _ => panic!("TODO"),
        }
    }

    pub fn set_reg(&mut self, reg: u32, val: u32) {
        match reg {
            0..=12 => self.r[reg as usize] = val,
            13 => self.r13_sp = val,
            14 => self.r14 = val,
            15 => self.r15_pc = val,
            _ => panic!("TODO"),
        }
    }

    pub fn set_cpsr(&mut self, flag: PSRFlags, set: bool) {
        match flag {
            PSRFlags::Negative => {
                if set {
                    self.cpsr |= 1 << 31;
                } else {
                    self.cpsr &= !(1 << 31);
                }
            }
            PSRFlags::Zero => {
                if set {
                    self.cpsr |= 1 << 30;
                } else {
                    self.cpsr &= !(1 << 30);
                }
            }
            PSRFlags::Carry => {
                if set {
                    self.cpsr |= 1 << 29;
                } else {
                    self.cpsr &= !(1 << 29);
                }
            }
            PSRFlags::Overflow => {
                if set {
                    self.cpsr |= 1 << 28;
                } else {
                    self.cpsr &= !(1 << 28);
                }
            }
            _ => panic!("TODO"),
        }
    }


    pub fn check_cond(&self, cond: u32) -> bool {
        let n: bool = (self.cpsr >> 31) & 0x01 == 1;
        let z: bool = (self.cpsr >> 30) & 0x01 == 1;
        let _c: bool = (self.cpsr >> 29) & 0x01 == 1;
        let v: bool = (self.cpsr >> 28) & 0x01 == 1;

        // println!("Got {}, {}, {}, {}", n, z, c, v);
        match cond {
            0x0 => z,
            0x1 => !z,
            0xB => n != v,
            0xE => true,
            0xF => false,
            _ => panic!("Condition {:#2X} not implemented!", cond),
        }
    }
}
