use crate::gba::bus::{self};
use crate::gba::cpu::arm7tdmi::{self, PSRFlags};

#[derive(Clone, Copy)]
pub struct ArmInstruction {
    pub name: Instruction,
    pub handler: fn(&mut arm7tdmi::Processor, &mut bus::Bus, u32),
}

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    BranchAndLink,
    BranchAndExchange,
    SoftwareInterrupt,
    DataProcessing,
    Multiply,
    MultiplyLong,
    StatusTransfer,
    SingleDataTransfer,
    HalfWordSignedTransfer,
    BlockDataTransfer,
    SingleDataSwap,
    CoprocessorInstructions,
    Unknown,
}

pub const BRANCH_FORMAT: u32 = 0b1010_0000_0000;
pub const BRANCH_MASK: u32 = 0b1110_0000_0000;

pub const HALFWORD_DT_IMM_FORMAT: u32 = 0b0000_0000_1001;
pub const HALFWORD_DT_IMM_MASK: u32 = 0b1110_0100_1001;

pub const HALFWORD_DT_REG_FORMAT: u32 = 0b0000_0100_1001;
pub const HALFWORD_DT_REG_MASK: u32 = 0b1110_0100_1001;

pub const DATA_PROCESSING_FORMAT: u32 = 0b0000_0000_0000;
pub const DATA_PROCESSING_MASK: u32 = 0b1100_0000_0000;

pub const ARM_UNKNOWN: ArmInstruction = ArmInstruction {
    name: Instruction::Unknown,
    handler: ArmInstruction::unknown_instruction,
};

pub const ARM_DATA_PROC: ArmInstruction = ArmInstruction {
    name: Instruction::DataProcessing,
    handler: ArmInstruction::data_processing,
};

pub const ARM_MULTIPLY: ArmInstruction = ArmInstruction {
    name: Instruction::Multiply,
    handler: ArmInstruction::multiply,
};

pub const ARM_MULTIPLY_LONG: ArmInstruction = ArmInstruction {
    name: Instruction::MultiplyLong,
    handler: ArmInstruction::multiply_long,
};

pub const ARM_STATUS_TRANSFER: ArmInstruction = ArmInstruction {
    name: Instruction::StatusTransfer,
    handler: ArmInstruction::status_transfer,
};

pub const ARM_SINGLE_DATA_TRANSFER: ArmInstruction = ArmInstruction {
    name: Instruction::SingleDataTransfer,
    handler: ArmInstruction::single_data_transfer,
};

pub const ARM_HALF_SIGNED_TRANSFER: ArmInstruction = ArmInstruction {
    name: Instruction::HalfWordSignedTransfer,
    handler: ArmInstruction::halfword_signed_transfer,
};

pub const ARM_BLOCK_DATA_TRANSFER: ArmInstruction = ArmInstruction {
    name: Instruction::BlockDataTransfer,
    handler: ArmInstruction::block_data_transfer,
};

pub const ARM_BRANCH_AND_EXCHANGE: ArmInstruction = ArmInstruction {
    name: Instruction::BranchAndExchange,
    handler: ArmInstruction::branch_and_exchange,
};

pub const ARM_BRANCH_AND_LINK: ArmInstruction = ArmInstruction {
    name: Instruction::BranchAndLink,
    handler: ArmInstruction::branch_and_link,
};

impl ArmInstruction {
    pub fn unknown_instruction(_cpu: &mut arm7tdmi::Processor, _bus: &mut bus::Bus, opcode: u32) {
        panic!("Error:Unknown instruction! Got {:#2X}\n", opcode);
    }

    pub fn data_processing(cpu: &mut arm7tdmi::Processor, bus: &mut bus::Bus, instr: u32) {
        log::info!("Data Processing Instruction");

        let opcode: u32 = (instr >> 21) & 0x0F;
        // TODO: Check set flags
        let set_flags = (instr >> 20) & 0x01 == 1;
        let is_imm = (instr >> 25) & 0x01 == 1;

        let reg_op1 = (instr >> 16) & 0x0F;
        let reg_dest = (instr >> 12) & 0x0F;

        log::debug!("Registers: Rn {}, Rd {}", reg_op1, reg_dest);

        // 1 means shift by register, 0 means immediate
        let shift_type = (instr >> 4) == 1;
        let op1: u32;
        let op2: u32;
        let mut carry = (cpu.regs.cpsr >> 29) & 0x01;

        if is_imm {
            log::info!("Immediate as 2nd operand");

            let imm = instr & 0xFF;
            let shift = ((instr >> 8) & 0xF) * 2;
            if shift != 0 {
                // Need to get carry from here?
                carry = (imm >> (shift - 1)) & 0x01;
                op2 = imm.rotate_right(shift);
            } else {
                op2 = imm;
            }

            op1 = cpu.regs.get_reg(reg_op1);
            log::debug!("Op1 is reg {} with value {:#2X}", reg_op1, op1);
            log::debug!("Op2 is immediate shift: {:#2X}", op2);
        } else {
            log::info!("Register as 2nd operand");

            if shift_type {
                // Register
                panic!("TODO");
            } else {
                // Immediate
                log::info!("Shifting by immediate");
                let imm = (instr >> 7) & 0x1F;
                let rm = (instr) & 0xF;

                log::debug!("Imm {:#2X}, Rm register {}", imm, rm);

                op1 = cpu.regs.get_reg(reg_op1);
                op2 = cpu.regs.get_reg(rm) << imm;
            }
        }

        match opcode {
            // ADD
            0x04 => cpu.regs.set_reg(reg_dest, op1 + op2),
            // TST
            0x08 => {
                let res: u32 = op1 & op2;
                log::debug!("TST: Result is {} with hex {:#2X}", res, res);
                cpu.regs.set_cpsr(PSRFlags::Negative, (res >> 31) == 1);
                cpu.regs.set_cpsr(PSRFlags::Zero, res == 0);
                // Carry bit is the carry flag of the shift operation
                // in the TST operation.
                cpu.regs.set_cpsr(PSRFlags::Carry, carry == 1);
                // Don't ask me how this works please :D
                cpu.regs
                    .set_cpsr(PSRFlags::Overflow, (((op1 ^ op2) & (op1 ^ res)) >> 31) == 1);
            }
            // CMP
            0x0A => {
                let res = op1.wrapping_sub(op2);
                log::debug!("CMP: Result is {} with hex {:#2X}", res, res);

                cpu.regs.set_cpsr(PSRFlags::Negative, (res >> 31) == 1);
                cpu.regs.set_cpsr(PSRFlags::Zero, res == 0);
                cpu.regs.set_cpsr(PSRFlags::Carry, op1 >= op2);
                // Don't ask me how this works please :D
                cpu.regs
                    .set_cpsr(PSRFlags::Overflow, (((op1 ^ op2) & (op1 ^ res)) >> 31) == 1);
            }
            // MOV
            0x0D => {
                cpu.regs.set_reg(reg_dest, op2);
                log::debug!("MOV: Setting reg {} to result {} with hex {:#2X}", reg_dest, op2, op2);

            }
            _ => panic!("Opcode {:#2X} not implemented for data proc!", opcode),
        }

        if reg_dest == 15 {
            log::debug!("Reloading pipeline in Data Processing!");
            cpu.reload_arm_pipeline(bus);
        }

        log::debug!(
            "Dest reg is {} with {:#2X}",
            reg_dest,
            cpu.regs.get_reg(reg_dest)
        );
    }

    pub fn multiply(_cpu: &mut arm7tdmi::Processor, _bus: &mut bus::Bus, _instr: u32) {
        panic!("MULTIPLY: TODO!");
    }

    pub fn multiply_long(_cpu: &mut arm7tdmi::Processor, _bus: &mut bus::Bus, _instr: u32) {
        panic!("MULTIPLY LONG: TODO!");
    }

    pub fn status_transfer(_cpu: &mut arm7tdmi::Processor, _bus: &mut bus::Bus, instr: u32) {
        let is_imm = (instr >> 25) & 0x01 == 1;
        // 0=CPSR, 1=SPSR_<current mode>
        let psr = (instr >> 22) & 0x01 == 1;

    }

    pub fn single_data_transfer(cpu: &mut arm7tdmi::Processor, bus: &mut bus::Bus, instr: u32) {
        let is_imm = (instr >> 25) & 0x01 == 0;
        let pre = (instr >> 24) & 0x01 == 1;
        let add = (instr >> 23) & 0x01 == 1;
        let byte = (instr >> 22) & 0x01 == 1;
        let writeback = (instr >> 21) & 0x01 == 1 || !pre;
        let load = (instr >> 20) & 0x01 == 1;

        let reg_base = (instr >> 16) & 0x0F;
        let reg_dest = (instr >> 12) & 0x0F;

        let mut address = cpu.regs.r[reg_base as usize];

        let offset = if is_imm {
            log::info!("Immediate offset!");
            //println!("IMM");
            instr & 0xFFF
        } else {
            log::info!("Register offset!");
            //println!("REG");
            let shift_amt = (instr >> 7) & 0x1F;
            let shift_type = (instr >> 5) & 0b11;
            let reg_offset = instr & 0xF;
            //log::debug!("Shift is {:#2X}", offset);
            //log::debug!("Reg is {:#2X}", offset);
            //log::debug!("Offset is {:#2X}", offset);

            // Might need to abstract this out later!
            match shift_type {
                // LSL
                0 => cpu.regs.get_reg(reg_offset) << shift_amt,
                // LSR
                1 => panic!("LSR NOT DONE"),
                // ASR
                2 => panic!("ASR NOT DONE"),
                // ROR
                3 => panic!("ROR NOT DONE"),
                _ => panic!("Incorrect shift type got {}", shift_type),
            }
        };

        log::debug!("Offset is {:#2X}", offset);

        if pre {
            log::info!("Pre indexing detected");
            if add {
                address += offset;
            } else {
                address -= offset;
            }
            log::debug!("Address is {:#2X}", address);
        }

        if load {
            // LDR
            log::info!("LDR operation");
            if byte {
                let value = bus.read_byte(address);
                cpu.regs.set_reg(reg_dest, u32::from(value));
                log::info!("Setting {} to {}", reg_dest, value);
            } else {
                panic!("LDR not implemented!");
            }
        } else {
            log::info!("STR operation");
            if byte {
                panic!("STRB not implemented!");
            } else {
                bus.write_word(address, cpu.regs.get_reg(reg_dest));
            }
        }

        if writeback || !pre {
            let base = cpu.regs.get_reg(reg_base);
            cpu.regs.set_reg(reg_base, base + offset);
            log::debug!("Setting reg {} to {:#2X}", reg_base, base + offset);
        }
    }

    pub fn halfword_signed_transfer(cpu: &mut arm7tdmi::Processor, bus: &mut bus::Bus, instr: u32) {
        log::info!("Halfword Signed Transfer Instruction");
        let pre = (instr >> 24 & 0x01) == 1;
        let add = (instr >> 23 & 0x01) == 1;
        let is_imm = (instr >> 22 & 0x01) == 1;
        let writeback = (instr >> 21 & 0x01) == 1;
        let load = (instr >> 20 & 0x01) == 1;

        let reg_base = (instr >> 16) & 0x0F;
        let reg_dest = (instr >> 12) & 0x0F;

        let mut address = cpu.regs.r[reg_base as usize];

        let offset = if is_imm {
            log::info!("Offset is immediate");
            let res = (instr & 0x0F) | ((instr >> 4) & 0xF0);
            log::debug!("Offset is {}", res);
            res
        } else {
            log::info!("Offset is register");
            let res = cpu.regs.get_reg(instr & 0x0F);
            log::debug!("Offset is {}", res);
            res
        };

        if pre {
            log::info!("Pre indexing detected");
            if add {
                address += offset;
            } else {
                address -= offset;
            }
            log::debug!("Address is {:#2X}", address);
        }

        let opcode = (instr >> 5) & 0b11;
        match opcode {
            0 => panic!("Should not have happened! Reserved for SWP"),
            1 => {
                if load {
                    let value = bus.read_half(address);
                    cpu.regs.set_reg(reg_dest, value);
                    log::debug!("Setting reg {} to {:#2X}", reg_dest, value);
                } else {
                    let value = cpu.regs.get_reg(reg_dest);
                    bus.write_half(address, value);
                    log::debug!("Writing to addr {:#2X} with value {:#2X}", address, value);
                }
            }
            _ => panic!("Unimplemented got {}", opcode),
        }

        if writeback || !pre {
            let base = cpu.regs.get_reg(reg_base);
            cpu.regs.set_reg(reg_base, base + offset);
            log::debug!("Writing to reg {} with value {:#2X}", reg_base, base + offset);
        }
    }

    pub fn block_data_transfer(cpu: &mut arm7tdmi::Processor, bus: &mut bus::Bus, instr: u32) {
        let mut pre = (instr >> 24) & 0x01 == 1;
        let add = (instr >> 23) & 0x01 == 1;
        // What does this do?
        let psr = (instr >> 22) & 0x01 == 1;
        let writeback = (instr >> 21) & 0x01 == 1;
        let load = (instr >> 20) & 0x01 == 1;

        let reg_base = (instr >> 16) & 0x0F;
        let reg_list = instr & 0xFF_FF;

        let mut address = cpu.regs.get_reg(reg_base);
        let mut offset = 0;

        // I think I have to do some special stuff for PC here too...
        // Offset is 4 * the number of registers per https://datasheets.chipdb.org/ARM/arm.pdf
        if reg_list == 0 {
            // This means we are modifying the PC R15?
            panic!("TODO: Block data transfer for when reg list is empty");
        } else {
            let i = 0b1;
            for r in 0..15 {
                // Get flipped reg list and test if bit r is 1
                // If so, that means that reg r was not specified in list.
                if (!reg_list & (i << r)) >> r == 1 {
                    continue;
                }
                offset += 4;
            }
        }

        // Note that stack grows up?
        let base_new = if add {
            address + offset
        } else {
            // If we are decrementing, we need to account
            // for the fact that the stack grows up from
            // the next address. So, this should mean that
            // the order in which we increment the address
            // is flipped, which is why we flip the pre bit,
            // if we are decrementing?
            pre = !pre;
            address = address.wrapping_sub(offset);
            address
        };

        let i = 0b1;
        for r in 0..15 {
            // Get flipped reg list and test if bit r is 1
            // If so, that means that reg r was not specified in list.
            if (!reg_list & (i << r)) >> r == 1 {
                continue;
            }
            if pre {
                address += 4;
            }

            if load {
                let value = bus.read_word(address);
                cpu.regs.set_reg(r, value);
            } else {
                bus.write_word(address, cpu.regs.get_reg(r));
            }

            if !pre {
                address += 4;
            }
        }

        // TODO: Do I need to check the base register?
        if writeback {
            cpu.regs.set_reg(reg_base, base_new);
        }
    }

    pub fn branch_and_link(cpu: &mut arm7tdmi::Processor, bus: &mut bus::Bus, instr: u32) {
        log::info!("Branch instruction");
        // TODO: Implement the link part for this too
        let link = (instr >> 24) & 0b01 == 1;
        let mut imm: i32 = (instr & 0xFF_FF_FF) as i32;

        // This sign extends the immediate.
        imm <<= 8;
        imm >>= 6;

        // Is this right?
        if link {
            cpu.regs.set_reg(14, cpu.regs.r15_pc - 4);
        }

        // Do not add +8 because of the prefetch operation?
        // imm += 4;
        cpu.regs.r15_pc = cpu.regs.r15_pc.wrapping_add(imm as u32);
        cpu.reload_arm_pipeline(bus);
    }

    pub fn branch_and_exchange(_cpu: &mut arm7tdmi::Processor, _bus: &mut bus::Bus, _instr: u32) {
        panic!("BRANCH AND EXCHANGE TODO!");
    }

    // TODO: Figure out a better way to possibly separate out
    // this logic with setting the status flags.
    fn add(op1: u32, op2: u32, set_flags: bool) -> u32 {
        if set_flags {
            panic!("set flags not implemented");
        }
        op1 + op2
    }

    fn cmp(op1: u32, op2: u32, set_flags: bool) -> u32 {
        if set_flags {
            panic!("set flags not implemented");
        }
        op1 + op2
    }
}
