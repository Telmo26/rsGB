mod proc;
mod instruction;
mod registers;
mod stack;
pub mod interrupts;
mod debug;

use crate::{
    utils::{bit_set, BIT_IGNORE}, Devices, Interconnect
};

pub use instruction::*;
use registers::*;

pub struct CPU {
    pub registers: CpuRegisters,

    // Current fetch
    fetched_data: u16,
    mem_dest: u16,
    dest_is_mem: bool,
    pub curr_opcode: u8,
    pub curr_inst: Instruction,

    halted: bool,
    stepping: bool,

    int_master_enabled: bool,
    enabling_ime: bool,
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

            int_master_enabled: false,
            enabling_ime: false,
        }
    }

    pub fn step(&mut self, mut dev: Devices) -> bool {
        if !self.halted {
            let previous_pc = self.registers.pc;

            self.fetch_instruction(&mut dev);
            dev.incr_cycle(1);
            self.fetch_data(&mut dev);

            if let Some(debugger) = dev.debugger {
                debugger.debug_info(self, dev.bus.as_mut().unwrap(), *dev.ticks, previous_pc);
                // debugger.gameboy_doctor(self, dev.bus.as_mut().unwrap(), previous_pc);
            }

            self.execute(&mut dev, self.curr_inst.in_type);
        } else {
            dev.incr_cycle(1);
            if self.get_int_flags(&dev) != 0 {
                self.halted = false;
            }
        }

        if self.int_master_enabled {
            self.handle_interrupts(&mut dev);
            self.enabling_ime = false;
        }

        if self.enabling_ime {
            self.int_master_enabled = true;
        }
        true
    }

    fn fetch_instruction(&mut self, dev: &mut Devices) {
        self.curr_opcode = dev.bus.as_mut().unwrap().read(self.registers.pc);
        self.curr_inst = Instruction::from_opcode(self.curr_opcode);

        self.registers.pc += 1;
    }

    fn fetch_data(&mut self, dev: &mut Devices) {
        self.mem_dest = 0;
        self.dest_is_mem = false;

        match self.curr_inst.mode {
            AddrMode::IMP => (),
            AddrMode::R => self.fetched_data = self.registers.read(self.curr_inst.reg_1),
            AddrMode::R_R => self.fetched_data = self.registers.read(self.curr_inst.reg_2),
            AddrMode::R_D8 => {
                self.fetched_data = dev.bus.as_mut().unwrap().read(self.registers.pc) as u16;
                dev.incr_cycle(1);
                self.registers.pc += 1;
            },
            AddrMode::D16 | AddrMode::R_D16 => {
                let low: u8 = dev.bus.as_mut().unwrap().read(self.registers.pc);
                dev.incr_cycle(1);
                self.registers.pc += 1;

                let high: u8 = dev.bus.as_mut().unwrap().read(self.registers.pc);
                dev.incr_cycle(1);
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

                self.fetched_data = dev.bus.as_mut().unwrap().read(addr) as u16;
                dev.incr_cycle(1);
            }
            AddrMode::R_HLD => {
                self.fetched_data = dev.bus.as_mut().unwrap().read(self.registers.read(self.curr_inst.reg_2)) as u16;
                dev.incr_cycle(1);
                self.registers.set(RegType::HL, self.registers.read(RegType::HL).wrapping_sub(1));
            }
            AddrMode::R_HLI => {
                self.fetched_data = dev.bus.as_mut().unwrap().read(self.registers.read(self.curr_inst.reg_2)) as u16;
                dev.incr_cycle(1);
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
                let address = dev.bus.as_ref().unwrap().read(self.registers.pc) as u16 | 0xFF00;
                self.fetched_data = dev.bus.as_ref().unwrap().read(address) as u16;
                dev.incr_cycle(1);
                self.registers.pc += 1;
            }
            AddrMode::A8_R => {
                self.fetched_data = self.registers.read(self.curr_inst.reg_2);
                self.mem_dest = dev.bus.as_ref().unwrap().read(self.registers.pc) as u16 | 0xFF00;
                self.dest_is_mem = true;
                dev.incr_cycle(1);
                self.registers.pc += 1;
            }
            AddrMode::HL_SP => {
                self.fetched_data = dev.bus.as_mut().unwrap().read(self.registers.pc) as u16;
                dev.incr_cycle(1);
                self.registers.pc += 1;
            }
            AddrMode::D8 => {
                self.fetched_data = dev.bus.as_mut().unwrap().read(self.registers.pc) as u16;
                dev.incr_cycle(1);
                self.registers.pc += 1;
            }
            AddrMode::D16_R | AddrMode::A16_R => {
                let low: u8 = dev.bus.as_mut().unwrap().read(self.registers.pc);
                dev.incr_cycle(1);

                let high: u8 = dev.bus.as_mut().unwrap().read(self.registers.pc + 1);
                dev.incr_cycle(1);

                self.mem_dest = (high as u16) << 8 | low as u16;
                self.dest_is_mem = true;

                self.registers.pc += 2;
                self.fetched_data = self.registers.read(self.curr_inst.reg_2);
            }
            AddrMode::MR_D8 => {
                self.fetched_data = dev.bus.as_mut().unwrap().read(self.registers.pc) as u16;
                dev.incr_cycle(1);
                self.registers.pc += 1;
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;
            }
            AddrMode::MR => {
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;
                self.fetched_data = dev.bus.as_mut().unwrap().read(self.registers.read(self.curr_inst.reg_1)) as u16;
                dev.incr_cycle(1);
            }
            AddrMode::R_A16 => {
                let low: u8 = dev.bus.as_ref().unwrap().read(self.registers.pc);
                dev.incr_cycle(1);

                let high: u8 = dev.bus.as_ref().unwrap().read(self.registers.pc + 1);
                dev.incr_cycle(1);

                let addr = (high as u16) << 8 | low as u16;

                self.registers.pc += 2;
                self.fetched_data = dev.bus.as_ref().unwrap().read(addr) as u16;
                dev.incr_cycle(1);
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

    fn get_int_flags(&self, dev: &Devices) -> u8 {
        dev.bus.as_ref().unwrap().read(0xFF0F)
    }

    fn set_int_flags(&mut self, dev: &mut Devices, value: u8) {
        dev.bus.as_mut().unwrap().write(0xFF0F, value);
    }
}