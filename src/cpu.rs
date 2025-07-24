use std::sync::MutexGuard;

mod cpu_proc;
mod instruction;

use crate::{
    EmuContext, Interconnect, utils::{bit_set, BIT_IGNORE}
};

use cpu_proc::cpu_proc;
use instruction::*;

struct CpuRegisters {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
}

impl CpuRegisters {
    fn new() -> CpuRegisters {
        CpuRegisters { a: 0x01, f: 0, b: 0, c: 0, d: 0, e: 0, h: 0, l: 0, pc: 0x100, sp: 0 }
    }

    fn read(&self, register: RegType) -> u16 {
        match register {
            RegType::NONE => 0,

            RegType::A => self.a as u16,
            RegType::F => self.f as u16,
            RegType::B => self.b as u16,
            RegType::C => self.c as u16,
            RegType::D => self.d as u16,
            RegType::E => self.e as u16,
            RegType::H => self.h as u16,
            RegType::L => self.l as u16,

            RegType::AF => (self.a as u16) << 8 | self.f as u16,
            RegType::BC => (self.b as u16) << 8 | self.c as u16,
            RegType::DE => (self.d as u16) << 8 | self.e as u16,
            RegType::HL => (self.h as u16) << 8 | self.l as u16,

            RegType::SP => self.sp as u16,
            RegType::PC => self.pc as u16,
        }
    }

    fn set(&mut self, register: RegType, value: u16) {
        match register {
            RegType::NONE => (),

            RegType::A => self.a = value as u8,
            RegType::F => self.f = value as u8,
            RegType::B => self.b = value as u8,
            RegType::C => self.c = value as u8,
            RegType::D => self.d = value as u8,
            RegType::E => self.e = value as u8,
            RegType::H => self.h = value as u8,
            RegType::L => self.l = value as u8,

            RegType::AF => {
                self.a = ((value & 0xFF00) >> 8) as u8;
                self.f = value as u8;
            },
            RegType::BC => {
                self.b = ((value & 0xFF00) >> 8) as u8;
                self.c = value as u8;
            },
            RegType::DE => {
                self.d = ((value & 0xFF00) >> 8) as u8;
                self.e = value as u8;
            },
            RegType::HL => {
                self.h = ((value & 0xFF00) >> 8) as u8;
                self.l = value as u8;
            },
            RegType::SP => self.sp = value,
            RegType::PC => self.pc = value,
        }
    }
}

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
            self.execute(ctx);
        }

        true
    }

    fn fetch_instruction(&mut self, bus: &mut Interconnect) {
        self.curr_opcode = bus.read(self.registers.pc);
        self.curr_inst = Instruction::from_opcode(self.curr_opcode);

        println!("PC: {0:04X} \t {1:?} ({2:X} {3:X} {4:X}) A: {5} B: {6} C: {7}", 
        self.registers.pc, self.curr_inst.in_type, self.curr_opcode, 
        bus.read(self.registers.pc + 1), bus.read(self.registers.pc+ 2),
        self.registers.a, self.registers.b, self.registers.c,
    );

        self.registers.pc += 1;
    }

    fn fetch_data(&mut self, bus: &mut Interconnect, ctx: &mut MutexGuard<'_, EmuContext>) {
        self.mem_dest = 0;
        self.dest_is_mem = false;

        match self.curr_inst.mode {
            AddrMode::IMP => (),
            AddrMode::R => self.fetched_data = self.registers.read(self.curr_inst.reg_1),
            AddrMode::RR => self.fetched_data = self.registers.read(self.curr_inst.reg_2),
            AddrMode::RD8 => {
                self.fetched_data = bus.read(self.registers.pc) as u16;
                ctx.incr_cycle(1);
                self.registers.pc += 1;
            },
            AddrMode::D16 | AddrMode::RD16 => {
                let low: u8 = bus.read(self.registers.pc);
                ctx.incr_cycle(1);

                let high: u8 = bus.read(self.registers.pc + 1);
                ctx.incr_cycle(1);

                self.fetched_data = (high as u16) << 8 | low as u16;

                self.registers.pc += 2;
            }
            AddrMode::MRR => {
                self.fetched_data = self.registers.read(self.curr_inst.reg_2);
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;

                if self.curr_inst.reg_2 == RegType::C {
                    self.mem_dest |= 0xFF00;
                }
            }
            AddrMode::RMR => {
                let mut addr = self.registers.read(self.curr_inst.reg_2);
                if self.curr_inst.reg_2 == RegType::C {
                    addr |= 0xFF00;
                }

                self.fetched_data = bus.read(addr) as u16;
                ctx.incr_cycle(1);
            }
            AddrMode::RHLD => {
                self.fetched_data = bus.read(self.registers.read(self.curr_inst.reg_2)) as u16;
                ctx.incr_cycle(1);
                self.registers.set(RegType::HL, self.registers.read(RegType::HL) - 1);
            }
            AddrMode::RHLI => {
                self.fetched_data = bus.read(self.registers.read(self.curr_inst.reg_2)) as u16;
                ctx.incr_cycle(1);
                self.registers.set(RegType::HL, self.registers.read(RegType::HL) + 1);
            }
            AddrMode::HLDR => {
                self.fetched_data = self.registers.read(self.curr_inst.reg_2);
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;
                self.registers.set(RegType::HL, self.registers.read(RegType::HL) - 1);
            }
            AddrMode::HLIR => {
                self.fetched_data = self.registers.read(self.curr_inst.reg_2);
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;
                self.registers.set(RegType::HL, self.registers.read(RegType::HL) + 1);
            }
            AddrMode::RA8 => {
                self.fetched_data = bus.read(self.registers.pc) as u16;
                ctx.incr_cycle(1);
                self.registers.pc += 1;
            }
            AddrMode::A8R => {
                self.mem_dest = bus.read(self.registers.pc) as u16 | 0xFF00;
                self.dest_is_mem = true;
                ctx.incr_cycle(1);
            }
            AddrMode::HLSPR => {
                self.fetched_data = bus.read(self.registers.pc) as u16;
                ctx.incr_cycle(1);
                self.registers.pc += 1;
            }
            AddrMode::D8 => {
                self.fetched_data = bus.read(self.registers.pc) as u16;
                ctx.incr_cycle(1);
                self.registers.pc += 1;
            }
            AddrMode::D16R | AddrMode::A16R => {
                let low: u8 = bus.read(self.registers.pc);
                ctx.incr_cycle(1);

                let high: u8 = bus.read(self.registers.pc + 1);
                ctx.incr_cycle(1);

                self.mem_dest = (high as u16) << 8 | low as u16;
                self.dest_is_mem = true;

                self.registers.pc += 2;
                self.fetched_data = self.registers.read(self.curr_inst.reg_2);
            }
            AddrMode::MRD8 => {
                self.fetched_data = bus.read(self.registers.pc) as u16;
                ctx.incr_cycle(1);
                self.registers.pc += 1;
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;
            }
            AddrMode::MR => {
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;
                self.fetched_data = bus.read(self.registers.read(self.curr_inst.reg_1)) as u16;
                ctx.incr_cycle(1);
            }
            AddrMode::RA16 => {
                let low: u8 = bus.read(self.registers.pc);
                ctx.incr_cycle(1);

                let high: u8 = bus.read(self.registers.pc + 1);
                ctx.incr_cycle(1);

                let addr = (high as u16) << 8 | low as u16;

                self.registers.pc += 2;
                self.fetched_data = bus.read(addr) as u16;
                ctx.incr_cycle(1);
            }

            // _ => println!("Unknown addressing mode: {0:?} ({1:X})", 
            //         self.curr_inst.mode, self.curr_opcode)
        }
    }

    fn execute(&mut self, ctx: &mut MutexGuard<'_, EmuContext>) {
        let mut process_instruction = cpu_proc(self.curr_inst.in_type);
        process_instruction(self, ctx)
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
