use std::sync::MutexGuard;

use crate::{cpu::instruction::{AddrMode, RegType}, interconnect::Interconnect, EmuContext};

use super::CPU;

impl CPU {
    pub fn gameboy_doctor(&self, bus: &mut Interconnect, ctx: &mut MutexGuard<'_, EmuContext>, previous_pc: u16) {
        let hl  = if self.curr_inst.mode == AddrMode::HLI_R || self.curr_inst.mode == AddrMode::R_HLI { 
            self.registers.read(RegType::HL) - 1
        } else if self.curr_inst.mode == AddrMode::HLD_R || self.curr_inst.mode == AddrMode::R_HLD {
            self.registers.read(RegType::HL) + 1
        } else {
            self.registers.read(RegType::HL)
        };

        println!( // GB Doctor
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            self.registers.a,
            self.registers.f,
            self.registers.b,
            self.registers.c,
            self.registers.d,
            self.registers.e,
            (hl & 0xFF00) >> 8,
            hl & 0x00FF,
            self.registers.sp,
            previous_pc,
            bus.read(previous_pc),
            bus.read(previous_pc + 1),
            bus.read(previous_pc + 2),
            bus.read(previous_pc + 3),
        );
    }

    pub fn debug_info(&self, bus: &mut Interconnect, ctx: &mut MutexGuard<'_, EmuContext>, previous_pc: u16) {
        let hl  = if self.curr_inst.mode == AddrMode::HLI_R || self.curr_inst.mode == AddrMode::R_HLI { 
            self.registers.read(RegType::HL) - 1
        } else if self.curr_inst.mode == AddrMode::HLD_R || self.curr_inst.mode == AddrMode::R_HLD {
            self.registers.read(RegType::HL) + 1
        } else {
            self.registers.read(RegType::HL)
        };
        
        let flags = format!(
            "Flags : {}{}{}{}",
            if self.registers.f & 1 << 7 != 0 { 'Z' } else { '-' },
            if self.registers.f & 1 << 6 != 0 { 'N' } else { '-' },
            if self.registers.f & 1 << 5 != 0 { 'H' } else { '-' },
            if self.registers.f & 1 << 4 != 0 { 'C' } else { '-' },
        );

        let inst_part = format!(
            "Ticks: {:08X} PC: {:04X} \t {} ({:02X} {:02X} {:02X})",
            ctx.ticks,
            previous_pc, 
            self.curr_inst.to_str(self), 
            self.curr_opcode, 
            bus.read(previous_pc + 1), 
            bus.read(previous_pc + 2),
        );

        

        let reg_part = format!(
            "A: {:02X} BC: {:02X}{:02X} DE: {:02X}{:02X} HL: {:04X} SP: {:04X}", 
            self.registers.a, 
            self.registers.b, self.registers.c,
            self.registers.d, self.registers.e, 
            hl,
            self.registers.sp
        );

        println!("{:<35} {}", inst_part, reg_part);
        println!("{:<32} {}", "", flags);

        ctx.debugger.as_mut().unwrap().update(bus);
        ctx.debugger.as_ref().unwrap().print();
    }
}