use std::sync::MutexGuard;

mod proc;
mod instruction;
mod registers;
mod stack;
mod interrupts;

use crate::{
    EmuContext, Interconnect, utils::{bit_set, BIT_IGNORE}
};

use instruction::*;
use registers::*;
use interrupts::*;

pub struct CPU {
    registers: CpuRegisters,

    // Current fetch
    fetched_data: u16,
    mem_dest: u16,
    dest_is_mem: bool,
    curr_opcode: u8,
    curr_inst: Instruction,

    halted: bool,
    stepping: bool,

    int_master_enabled: bool,
    enabling_ime: bool,

    int_flags: u8,
}

impl CPU {
    pub fn new() -> CPU {
        let registers = CpuRegisters::new();
        CPU {
            registers,

            fetched_data: 0,
            mem_dest: 0,
            dest_is_mem: false,
            curr_opcode: 0,
            curr_inst: INSTRUCTIONS[0x00].unwrap(),

            halted: false,
            stepping: false,

            int_master_enabled: true,
            enabling_ime: false,

            int_flags: 0,
        }
    }

    pub fn step(&mut self, bus: &mut Interconnect, ctx: &mut MutexGuard<'_, EmuContext>) -> bool {
        
        if !self.halted {
            self.fetch_instruction(bus);
            self.fetch_data(bus, ctx);

            if ctx.debug {
                self.debugger(bus, ctx);
            }

            self.execute(bus, ctx, self.curr_inst.in_type);
        } else {
            ctx.incr_cycle();
            if self.int_flags != 0 {
                self.halted = false;
            }
        }

        if self.int_master_enabled {
            self.handle_interrupts(bus);
            self.enabling_ime = false;
        }

        if self.enabling_ime {
            self.int_master_enabled = true;
        }
        true
    }

    fn debugger(&self, bus: &mut Interconnect, ctx: &mut MutexGuard<'_, EmuContext>) {
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
            self.registers.pc, 
            self.curr_inst.to_str(self), 
            self.curr_opcode, 
            bus.read(self.registers.pc + 1), 
            bus.read(self.registers.pc+ 2),
        );

        let reg_part = format!(
            "A: {:02X} BC: {:02X}{:02X} DE: {:02X}{:02X} HL: {:02X}{:02X} SP: {:04X}", 
            self.registers.a, 
            self.registers.b, self.registers.c,
            self.registers.d, self.registers.e, 
            self.registers.h, self.registers.l,
            self.registers.sp
        );

        println!("{:<35} {}", inst_part, reg_part);
        println!("{:<32} {}", "", flags);

        ctx.debugger.as_mut().unwrap().update(bus);
        ctx.debugger.as_ref().unwrap().print();

    }

    fn fetch_instruction(&mut self, bus: &mut Interconnect) {
        self.curr_opcode = bus.read(self.registers.pc);
        self.curr_inst = Instruction::from_opcode(self.curr_opcode);

        self.registers.pc += 1;
    }

    fn fetch_data(&mut self, bus: &mut Interconnect, ctx: &mut MutexGuard<'_, EmuContext>) {
        self.mem_dest = 0;
        self.dest_is_mem = false;

        match self.curr_inst.mode {
            AddrMode::IMP => (),
            AddrMode::R => self.fetched_data = self.registers.read(self.curr_inst.reg_1),
            AddrMode::R_R => self.fetched_data = self.registers.read(self.curr_inst.reg_2),
            AddrMode::R_D8 => {
                self.fetched_data = bus.read(self.registers.pc) as u16;
                ctx.incr_cycle();
                self.registers.pc += 1;
            },
            AddrMode::D16 | AddrMode::R_D16 => {
                let low: u8 = bus.read(self.registers.pc);
                ctx.incr_cycle();
                self.registers.pc += 1;

                let high: u8 = bus.read(self.registers.pc);
                ctx.incr_cycle();
                self.registers.pc += 1;

                self.fetched_data = (high as u16) << 8 | low as u16;
            }
            AddrMode::MR_R => {
                self.fetched_data = self.registers.read(self.curr_inst.reg_2);
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;

                if self.curr_inst.reg_2 == RegType::C {
                    self.mem_dest |= 0xFF00;
                }
            }
            AddrMode::R_MR => {
                let mut addr = self.registers.read(self.curr_inst.reg_2);
                if self.curr_inst.reg_2 == RegType::C {
                    addr |= 0xFF00;
                }

                self.fetched_data = bus.read(addr) as u16;
                ctx.incr_cycle();
            }
            AddrMode::R_HLD => {
                self.fetched_data = bus.read(self.registers.read(self.curr_inst.reg_2)) as u16;
                ctx.incr_cycle();
                self.registers.set(RegType::HL, self.registers.read(RegType::HL) - 1);
            }
            AddrMode::R_HLI => {
                self.fetched_data = bus.read(self.registers.read(self.curr_inst.reg_2)) as u16;
                ctx.incr_cycle();
                self.registers.set(RegType::HL, self.registers.read(RegType::HL) + 1);
            }
            AddrMode::HLD_R => {
                self.fetched_data = self.registers.read(self.curr_inst.reg_2);
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;
                self.registers.set(RegType::HL, self.registers.read(RegType::HL) - 1);
            }
            AddrMode::HLI_R => {
                self.fetched_data = self.registers.read(self.curr_inst.reg_2);
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;
                self.registers.set(RegType::HL, self.registers.read(RegType::HL) + 1);
            }
            AddrMode::R_A8 => {
                self.fetched_data = bus.read(self.registers.pc) as u16;
                ctx.incr_cycle();
                self.registers.pc += 1;
            }
            AddrMode::A8_R => {
                self.fetched_data = self.registers.read(self.curr_inst.reg_2);
                self.mem_dest = bus.read(self.registers.pc) as u16 | 0xFF00;
                self.dest_is_mem = true;
                ctx.incr_cycle();
                self.registers.pc += 1;
            }
            AddrMode::HL_SP => {
                let e = bus.read(self.registers.pc).cast_signed();
                self.fetched_data = self.registers.sp.wrapping_add_signed(e as i16);
                ctx.incr_cycle();
                self.registers.pc += 1;
            }
            AddrMode::D8 => {
                self.fetched_data = bus.read(self.registers.pc) as u16;
                ctx.incr_cycle();
                self.registers.pc += 1;
            }
            AddrMode::D16_R | AddrMode::A16_R => {
                let low: u8 = bus.read(self.registers.pc);
                ctx.incr_cycle();

                let high: u8 = bus.read(self.registers.pc + 1);
                ctx.incr_cycle();

                self.mem_dest = (high as u16) << 8 | low as u16;
                self.dest_is_mem = true;

                self.registers.pc += 2;
                self.fetched_data = self.registers.read(self.curr_inst.reg_2);
            }
            AddrMode::MR_D8 => {
                self.fetched_data = bus.read(self.registers.pc) as u16;
                ctx.incr_cycle();
                self.registers.pc += 1;
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;
            }
            AddrMode::MR => {
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;
                self.fetched_data = bus.read(self.registers.read(self.curr_inst.reg_1)) as u16;
                ctx.incr_cycle();
            }
            AddrMode::R_A16 => {
                let low: u8 = bus.read(self.registers.pc);
                ctx.incr_cycle();

                let high: u8 = bus.read(self.registers.pc + 1);
                ctx.incr_cycle();

                let addr = (high as u16) << 8 | low as u16;

                self.registers.pc += 2;
                self.fetched_data = bus.read(addr) as u16;
                ctx.incr_cycle();
            }
        }
    }

    fn z_flag(&self) -> bool {
        (self.registers.f & 0b10000000) >> 7 == 1
    }

    fn n_flag(&self) -> bool {
        (self.registers.f & 0b01000000) >> 7 == 1
    }

    fn h_flag(&self) -> bool {
        (self.registers.f & 0b00100000) >> 7 == 1
    }

    fn c_flag(&self) -> bool {
        (self.registers.f & 0b00010000) >> 7 == 1
    }

    fn check_cond(&self) -> bool {
        let z = self.z_flag();
        let c = self.c_flag();

        match self.curr_inst.cond {
            CondType::NONE => true,
            CondType::Z => z,
            CondType::NZ => !z,
            CondType::C => c,
            CondType::NC => !c
        }
    }

    fn set_flags(&mut self, z: u8, n: u8, h: u8, c: u8) {
        if z != BIT_IGNORE {
            bit_set(&mut self.registers.f, 7, z == 1);
        }

        if n != BIT_IGNORE {
            bit_set(&mut self.registers.f, 6, n == 1);
        }

        if h != BIT_IGNORE {
            bit_set(&mut self.registers.f, 5, h == 1);
        }

        if c != BIT_IGNORE {
            bit_set(&mut self.registers.f, 4, c == 1);
        }
    }

    fn get_int_flags(&self) -> u8 {
        self.int_flags
    }

    fn set_int_flags(&mut self, value: u8) {
        self.int_flags = value;
    }
}
