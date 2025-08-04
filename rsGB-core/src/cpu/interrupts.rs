use crate::Devices;

use super::{CPU, Interconnect};

#[derive(Clone, Debug)]
pub enum InterruptType {
    VBlank = 1,
    LcdStat = 2,
    Timer = 4,
    Serial = 8,
    Joypad = 16,
}

impl CPU {
    fn interrupt_handle(&mut self, dev: &mut Devices, address: u16) {
        self.push16(&mut dev.bus, self.registers.pc);
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
        let ie_register = dev.bus.get_ie_register();
        let it = interrupt_type as u8;

        if (if_register & it) != 0 && 
            (ie_register & it) != 0 {

            self.interrupt_handle(dev, address);

            self.set_int_flags(dev, if_register & !it);
            
            self.halted = false;
            self.int_master_enabled = false;
            
            return true
        }
        false
    }
}