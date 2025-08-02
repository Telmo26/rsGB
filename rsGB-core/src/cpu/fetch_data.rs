use super::CPU;
use crate::{cpu::{AddrMode, RegType}, Devices};

impl CPU {
    pub fn fetch_data(&mut self, dev: &mut Devices) {
        self.mem_dest = 0;
        self.dest_is_mem = false;

        match self.curr_inst.mode {
            AddrMode::IMP => (),
            AddrMode::R => self.fetched_data = self.registers.read(self.curr_inst.reg_1),
            AddrMode::R_R => self.fetched_data = self.registers.read(self.curr_inst.reg_2),
            AddrMode::R_D8 => {
                self.fetched_data = dev.bus.read(self.registers.pc) as u16;
                dev.incr_cycle(1);
                self.registers.pc += 1;
            },
            AddrMode::D16 | AddrMode::R_D16 => {
                let low: u8 = dev.bus.read(self.registers.pc);
                dev.incr_cycle(1);
                self.registers.pc += 1;

                let high: u8 = dev.bus.read(self.registers.pc);
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

                self.fetched_data = dev.bus.read(addr) as u16;
                dev.incr_cycle(1);
            }
            AddrMode::R_HLD => {
                self.fetched_data = dev.bus.read(self.registers.read(self.curr_inst.reg_2)) as u16;
                dev.incr_cycle(1);
                self.registers.set(RegType::HL, self.registers.read(RegType::HL).wrapping_sub(1));
            }
            AddrMode::R_HLI => {
                self.fetched_data = dev.bus.read(self.registers.read(self.curr_inst.reg_2)) as u16;
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
                let address = dev.bus.read(self.registers.pc) as u16 | 0xFF00;
                self.fetched_data = dev.bus.read(address) as u16;
                dev.incr_cycle(1);
                self.registers.pc += 1;
            }
            AddrMode::A8_R => {
                self.fetched_data = self.registers.read(self.curr_inst.reg_2);
                self.mem_dest = dev.bus.read(self.registers.pc) as u16 | 0xFF00;
                self.dest_is_mem = true;
                dev.incr_cycle(1);
                self.registers.pc += 1;
            }
            AddrMode::HL_SP => {
                self.fetched_data = dev.bus.read(self.registers.pc) as u16;
                dev.incr_cycle(1);
                self.registers.pc += 1;
            }
            AddrMode::D8 => {
                self.fetched_data = dev.bus.read(self.registers.pc) as u16;
                dev.incr_cycle(1);
                self.registers.pc += 1;
            }
            AddrMode::D16_R | AddrMode::A16_R => {
                let low: u8 = dev.bus.read(self.registers.pc);
                dev.incr_cycle(1);

                let high: u8 = dev.bus.read(self.registers.pc + 1);
                dev.incr_cycle(1);

                self.mem_dest = (high as u16) << 8 | low as u16;
                self.dest_is_mem = true;

                self.registers.pc += 2;
                self.fetched_data = self.registers.read(self.curr_inst.reg_2);
            }
            AddrMode::MR_D8 => {
                self.fetched_data = dev.bus.read(self.registers.pc) as u16;
                dev.incr_cycle(1);
                self.registers.pc += 1;
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;
            }
            AddrMode::MR => {
                self.mem_dest = self.registers.read(self.curr_inst.reg_1);
                self.dest_is_mem = true;
                self.fetched_data = dev.bus.read(self.mem_dest) as u16;
                dev.incr_cycle(1);
            }
            AddrMode::R_A16 => {
                let low: u8 = dev.bus.read(self.registers.pc);
                dev.incr_cycle(1);

                let high: u8 = dev.bus.read(self.registers.pc + 1);
                dev.incr_cycle(1);

                let addr = (high as u16) << 8 | low as u16;

                self.registers.pc += 2;
                self.fetched_data = dev.bus.read(addr) as u16;
                dev.incr_cycle(1);
            }
        }
    }
}