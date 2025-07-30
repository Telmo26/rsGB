use crate::Devices;

use super::{CPU, Interconnect};

#[derive(Clone)]
pub enum InterruptType {
    VBlank = 1,
    LcdStat = 2,
    Timer = 4,
    Serial = 8,
    Joypad = 16,
}

impl InterruptType {
    fn value(&self) -> u8 {
        self.clone() as u8
    }
}

impl CPU {
    fn interrupt_handle(&mut self, dev: &mut Devices, address: u16) {
        let bus = dev.bus.as_mut().unwrap();
        self.push16(bus, self.registers.pc);
        self.registers.pc = address;
    }

    pub fn handle_interrupts(&mut self, dev: &mut Devices) {
        if self.interrupt_check(dev, 0x40, InterruptType::VBlank) {

        } else if self.interrupt_check(dev, 0x48, InterruptType::LcdStat) {

        } else if self.interrupt_check(dev, 0x50, InterruptType::Timer) {

        } else if self.interrupt_check(dev, 0x58, InterruptType::Serial) {

        } else if self.interrupt_check(dev, 0x60, InterruptType::Joypad) {
            
        }
    }

    fn interrupt_check(&mut self, dev: &mut Devices, address: u16, interrupt_type: InterruptType) -> bool {
        let if_register = self.get_int_flags(dev);
        if (if_register & interrupt_type.value()) != 0 && 
            (dev.bus.as_ref().unwrap().get_ie_register() & interrupt_type.value()) != 0 {
            self.interrupt_handle(dev, address);

            self.set_int_flags(dev, if_register &!interrupt_type.value());
            
            self.halted = false;
            self.int_master_enabled = false;
            return true
        }
        false
    }
}