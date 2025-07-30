use std::sync::MutexGuard;

mod proc;
mod instruction;
mod registers;
mod stack;
mod interrupts;
mod debug;

use crate::{
    utils::{bit_set, BIT_IGNORE}, EmuContext, Interconnect
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
            let previous_pc = self.registers.pc;
            self.fetch_instruction(bus);
            self.fetch_data(bus, ctx);

            if ctx.debug {
                self.debug_info(bus, ctx, previous_pc);
                // self.gameboy_doctor(bus, ctx, previous_pc);
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

                if self.curr_inst.reg_1 == RegType::C {
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
                self.registers.set(RegType::HL, self.registers.read(RegType::HL).wrapping_sub(1));
            }
            AddrMode::R_HLI => {
                self.fetched_data = bus.read(self.registers.read(self.curr_inst.reg_2)) as u16;
                ctx.incr_cycle();
                self.registers.set(RegType::HL, self.registers.read(RegType::HL).wrapping_add(1));
            }
            AddrMode::HLD_R => {
                self.fetched_data = self.registers.read(self.curr_inst.reg_2);
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;
                self.registers.set(RegType::HL, self.registers.read(RegType::HL).wrapping_sub(1));
            }
            AddrMode::HLI_R => {
                self.fetched_data = self.registers.read(self.curr_inst.reg_2);
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;
                self.registers.set(RegType::HL, self.registers.read(RegType::HL).wrapping_add(1));
            }
            AddrMode::R_A8 => {
                let address = bus.read(self.registers.pc) as u16 | 0xFF00;
                self.fetched_data = bus.read(address) as u16;
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
        }
    }

    fn z_flag(&self) -> bool {
        (self.registers.f & 0b10000000) != 0
    }

    fn n_flag(&self) -> bool {
        (self.registers.f & 0b01000000) != 0
    }

    fn h_flag(&self) -> bool {
        (self.registers.f & 0b00100000) != 0
    }

    fn c_flag(&self) -> bool {
        (self.registers.f & 0b00010000) != 0
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


#[test]
fn test_rr_d() {
    let mut cpu = CPU::new();
    let mut bus = Interconnect::new();
    let mut ctx = EmuContext::new("test_roms/01-special.gb", true);

    cpu.fetched_data = 0x1A; // RR D

    // RR carry 1
    cpu.registers.d = 0b10110001;

    cpu.set_flags(0, 0, 0, 1);

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0xCB].unwrap().in_type);
    assert_eq!(cpu.registers.d, 0b11011000);
    assert_eq!(cpu.z_flag() as u8, 0);
    assert_eq!(cpu.n_flag() as u8, 0);
    assert_eq!(cpu.h_flag() as u8, 0);
    assert_eq!(cpu.c_flag() as u8, 1);

    // RR carry 0
    cpu.registers.d = 0b10110000;

    cpu.set_flags(0, 0, 0, 0);

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0xCB].unwrap().in_type);
    assert_eq!(cpu.registers.d, 0b01011000);
    assert_eq!(cpu.z_flag() as u8, 0);
    assert_eq!(cpu.n_flag() as u8, 0);
    assert_eq!(cpu.h_flag() as u8, 0);
    assert_eq!(cpu.c_flag() as u8, 0);
    
    // RR result 0
    cpu.registers.d = 0x01;

    cpu.set_flags(0, 0, 0, 0);

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0xCB].unwrap().in_type);

    assert_eq!(cpu.registers.d, 0x00);
    assert_eq!(cpu.z_flag() as u8, 1);
    assert_eq!(cpu.n_flag() as u8, 0);
    assert_eq!(cpu.h_flag() as u8, 0);
    assert_eq!(cpu.c_flag() as u8, 1);
}

#[test]
fn test_rlc_b() {
    let mut cpu = CPU::new();
    let mut bus = Interconnect::new();
    let mut ctx = EmuContext::new("test_roms/01-special.gb", true);

    cpu.fetched_data = 0x00;

    // RLC carry 1
    cpu.registers.b = 0b10110001;
    cpu.set_flags(0, 0, 0, 0);

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0xCB].unwrap().in_type); // RLC B

    assert_eq!(cpu.registers.b, 0b01100011);
    assert_eq!(cpu.z_flag() as u8, 0);
    assert_eq!(cpu.c_flag() as u8, 1);

    // RLC carry 0
    cpu.registers.b = 0b00110001; // B1
    cpu.set_flags(0, 0, 0, 0);

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0xCB].unwrap().in_type); // RLC B

    assert_eq!(cpu.registers.b, 0b01100010);
    assert_eq!(cpu.z_flag() as u8, 0);
    assert_eq!(cpu.c_flag() as u8, 0);

    // RLC result 0
    cpu.registers.b = 0b00000000; // B1
    cpu.set_flags(0, 0, 0, 0);

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0xCB].unwrap().in_type); // RLC B

    assert_eq!(cpu.registers.b, 0);
    assert_eq!(cpu.z_flag() as u8, 1);
    assert_eq!(cpu.c_flag() as u8, 0);
}

#[test]
fn test_rlc_hl() {
    let mut cpu = CPU::new();
    let mut bus = Interconnect::new();
    let mut ctx = EmuContext::new("", true);

    cpu.fetched_data = 0x06;
    cpu.registers.set(RegType::HL, 0xC000); // Writing to High Ram

    // RLC with one
    cpu.registers.set_reg8(&mut bus, RegType::HL, 0b10110001);
    cpu.set_flags(0, 0, 0, 0);

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0xCB].unwrap().in_type);

    assert_eq!(cpu.registers.read_reg8(&mut bus, RegType::HL), 0b01100011);
    assert_eq!(cpu.z_flag() as u8, 0);
    assert_eq!(cpu.c_flag() as u8, 1);

    // RLC with zero
    cpu.registers.set_reg8(&mut bus, RegType::HL, 0b00110001);
    cpu.set_flags(0, 0, 0, 0);

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0xCB].unwrap().in_type);

    assert_eq!(cpu.registers.read_reg8(&mut bus, RegType::HL), 0b01100010);
    assert_eq!(cpu.z_flag() as u8, 0);
    assert_eq!(cpu.c_flag() as u8, 0);

    // RLC result 0
    cpu.registers.set_reg8(&mut bus, RegType::HL, 0b00000000);
    cpu.set_flags(0, 0, 0, 0);

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0xCB].unwrap().in_type);

    assert_eq!(cpu.registers.read_reg8(&mut bus, RegType::HL), 0);
    assert_eq!(cpu.z_flag() as u8, 1);
    assert_eq!(cpu.c_flag() as u8, 0);
}

#[test]
fn test_srl_b() {
    let mut cpu = CPU::new();
    let mut bus = Interconnect::new();
    let mut ctx = EmuContext::new("", true);

    cpu.fetched_data = 0x38; // SRL B

    // SRL carry 1
    cpu.registers.b = 0b10110001;

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0xCB].unwrap().in_type);

    assert_eq!(cpu.registers.b, 0b01011000);
    assert_eq!(cpu.z_flag() as u8, 0);
    assert_eq!(cpu.n_flag() as u8, 0);
    assert_eq!(cpu.h_flag() as u8, 0);
    assert_eq!(cpu.c_flag() as u8, 1);

    // SRLC carry 0
    cpu.registers.b = 0b10110000;

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0xCB].unwrap().in_type);

    assert_eq!(cpu.registers.b, 0b01011000);
    assert_eq!(cpu.z_flag() as u8, 0);
    assert_eq!(cpu.n_flag() as u8, 0);
    assert_eq!(cpu.h_flag() as u8, 0);
    assert_eq!(cpu.c_flag() as u8, 0);

    // SRL carry 1 result 0
    cpu.registers.b = 0b00000001;

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0xCB].unwrap().in_type);

    assert_eq!(cpu.registers.b, 0);
    assert_eq!(cpu.z_flag() as u8, 1);
    assert_eq!(cpu.n_flag() as u8, 0);
    assert_eq!(cpu.h_flag() as u8, 0);
    assert_eq!(cpu.c_flag() as u8, 1);

    // SRL carry 0 result 0
    cpu.registers.b = 0b00000000;

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0xCB].unwrap().in_type);

    assert_eq!(cpu.registers.b, 0);
    assert_eq!(cpu.z_flag() as u8, 1);
    assert_eq!(cpu.n_flag() as u8, 0);
    assert_eq!(cpu.h_flag() as u8, 0);
    assert_eq!(cpu.c_flag() as u8, 0);
}

#[test]
fn test_rra() {
    let mut cpu = CPU::new();
    let mut bus = Interconnect::new();
    let mut ctx = EmuContext::new("", true);

    // RR carry 1
    cpu.registers.a = 0b10110001;

    cpu.set_flags(0, 0, 0, 1);

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0x1F].unwrap().in_type);
    assert_eq!(cpu.registers.a, 0b11011000);
    assert_eq!(cpu.z_flag() as u8, 0);
    assert_eq!(cpu.n_flag() as u8, 0);
    assert_eq!(cpu.h_flag() as u8, 0);
    assert_eq!(cpu.c_flag() as u8, 1);

    // RR carry 0
    cpu.registers.a = 0b10110000;

    cpu.set_flags(0, 0, 0, 0);

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0x1F].unwrap().in_type);
    assert_eq!(cpu.registers.a, 0b01011000);
    assert_eq!(cpu.z_flag() as u8, 0);
    assert_eq!(cpu.n_flag() as u8, 0);
    assert_eq!(cpu.h_flag() as u8, 0);
    assert_eq!(cpu.c_flag() as u8, 0);
    
    // RR result 0
    cpu.registers.a = 0x01;

    cpu.set_flags(0, 0, 0, 0);

    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0x1F].unwrap().in_type);

    assert_eq!(cpu.registers.a, 0x00);
    assert_eq!(cpu.z_flag() as u8, 0);
    assert_eq!(cpu.n_flag() as u8, 0);
    assert_eq!(cpu.h_flag() as u8, 0);
    assert_eq!(cpu.c_flag() as u8, 1);
}

fn reference_daa(a: u8, n: bool, h: bool, c: bool) -> (u8, bool, bool) {
    let mut result = a;
    let mut carry = c;
    let mut adjust = 0;

    if !n {
        if h || (a & 0xF) > 9 {
            adjust |= 0x06;
        }
        if c || a > 0x99 {
            adjust |= 0x60;
            carry = true;
        }
        result = result.wrapping_add(adjust);
    } else {
        if h {
            adjust |= 0x06;
        }
        if c {
            adjust |= 0x60;
        }
        result = result.wrapping_sub(adjust);
    }

    let zero = result == 0;
    (result, zero, carry)
}

#[test]
fn test_daa_all_combinations() {
    for a in 0x00..=0xFF {
        for &n in &[false, true] {
            for &h in &[false, true] {
                for &c in &[false, true] {
                    // Setup CPU
                    let mut cpu = CPU::new();
                    cpu.registers.a = a;
                    cpu.set_flags(0, n as u8, h as u8, c as u8);

                    let mut bus = Interconnect::new();
                    let mut ctx = EmuContext::new("", true);

                    // Run your DAA
                    cpu.execute(&mut bus, &mut ctx, INSTRUCTIONS[0x27].unwrap().in_type);

                    // Expected result
                    let (expected_a, expected_z, expected_c) = reference_daa(a, n, h, c);

                    assert_eq!(
                        cpu.registers.a, expected_a,
                        "DAA failed: A={:02X} N={} H={} C={}",
                        a, n, h, c
                    );
                    assert_eq!(
                        cpu.z_flag(),
                        expected_z,
                        "Z flag incorrect: A={:02X} N={} H={} C={}",
                        a,
                        n,
                        h,
                        c
                    );
                    assert_eq!(
                        cpu.c_flag(),
                        expected_c,
                        "C flag incorrect: A={:02X} N={} H={} C={}",
                        a,
                        n,
                        h,
                        c
                    );
                }
            }
        }
    }
}