use std::sync::MutexGuard;

mod cpu_proc;
mod instruction;
mod registers;

use crate::{
    EmuContext, Interconnect, utils::{bit_set, BIT_IGNORE}
};

use cpu_proc::cpu_proc;
use instruction::*;
use registers::*;


pub struct Cpu {
    registers: CpuRegisters,

    // Current fetch
    fetched_data: u16,
    mem_dest: u16,
    dest_is_mem: bool,
    curr_opcode: u8,
    curr_inst: Instruction,

    halted: bool,
    stepping: bool,

    en_master_interrupt: bool
}

impl Cpu {
    pub fn new() -> Cpu {
        let registers = CpuRegisters::new();
        Cpu {
            registers,

            fetched_data: 0,
            mem_dest: 0,
            dest_is_mem: false,
            curr_opcode: 0,
            curr_inst: INSTRUCTIONS[0x00].unwrap(),

            halted: false,
            stepping: false,

            en_master_interrupt: true
        }
    }

    pub fn step(&mut self, bus: &mut Interconnect, ctx: &mut MutexGuard<'_, EmuContext>) -> bool {
        
        if !self.halted {
            self.fetch_instruction(bus);
            self.fetch_data(bus, ctx);
            self.execute(bus, ctx);
        }

        true
    }

    fn fetch_instruction(&mut self, bus: &mut Interconnect) {
        self.curr_opcode = bus.read(self.registers.pc);
        self.curr_inst = Instruction::from_opcode(self.curr_opcode);

        let inst_part = format!(
            "PC: {0:04X} \t {1:?} {2:?} ({3:02X} {4:02X} {5:02X})",
            self.registers.pc, 
            self.curr_inst.in_type, 
            self.curr_inst.mode, 
            self.curr_opcode, 
            bus.read(self.registers.pc + 1), 
            bus.read(self.registers.pc+ 2),
        );

        let reg_part = format!(
            "A: {0:02X} BC: {1:02X}{2:02X} DE: {3:02X}{4:02X} HL: {5:02X}{6:02X} SP: {7:04X}", 
            self.registers.a, 
            self.registers.b, self.registers.c,
            self.registers.d, self.registers.e, 
            self.registers.h, self.registers.l,
            self.registers.sp
        );

        println!("{:<35} {}", inst_part, reg_part);

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
                self.fetched_data = bus.read(self.registers.pc) as u16;
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

            // _ => println!("Unknown addressing mode: {0:?} ({1:X})", 
            //         self.curr_inst.mode, self.curr_opcode)
        }
    }

    fn execute(&mut self, bus: &mut Interconnect, ctx: &mut MutexGuard<'_, EmuContext>) {
        let mut process_instruction = cpu_proc(self.curr_inst.in_type);
        process_instruction(self, bus, ctx)
    }

    fn z_flag(&self) -> bool {
        (self.registers.f & 0b10000000) >> 7 == 1
    }

    fn c_flag(&self) -> bool {
        (self.registers.f & 0b00010000) >> 7 == 1
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
}
