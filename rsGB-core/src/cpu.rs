mod proc;
mod instruction;
mod registers;
mod stack;
pub mod interrupts;
mod fetch_data;

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

    pub fn step(&mut self, dev: &mut Devices) -> bool {
        if self.int_master_enabled {
            self.handle_interrupts(dev);
            self.enabling_ime = false;
        }

        if !self.halted {
            let previous_pc = self.registers.pc;

            self.fetch_instruction(dev);
            dev.incr_cycle(1);
            self.fetch_data(dev);

            if let Some(debugger) = &mut dev.debugger {
                debugger.debug_info(self, &mut dev.bus, dev.ticks, previous_pc);
                // debugger.gameboy_doctor(self, &mut dev.bus, previous_pc);
            }

            self.execute(dev, self.curr_inst.in_type);
        } else {
            dev.incr_cycle(1);
            if self.get_int_flags(&dev) != 0 {
                self.halted = false;
            }
        }

        if self.enabling_ime {
            self.int_master_enabled = true;
        }
        true
    }

    fn fetch_instruction(&mut self, dev: &mut Devices) {
        self.curr_opcode = dev.bus.read(self.registers.pc);
        self.curr_inst = Instruction::from_opcode(self.curr_opcode);

        self.registers.pc += 1;
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
        dev.bus.read(0xFF0F)
    }

    fn set_int_flags(&mut self, dev: &mut Devices, value: u8) {
        dev.bus.write(0xFF0F, value);
    }
}