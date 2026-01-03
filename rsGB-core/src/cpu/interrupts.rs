use crate::Devices;

use super::CPU;

#[derive(Clone, Copy, Debug)]
pub enum InterruptType {
    VBlank  = 0b0000_0001,
    LcdStat = 0b0000_0010,
    Timer   = 0b0000_0100,
    Serial  = 0b0000_1000,
    Joypad  = 0b0001_0000,
}

impl CPU {
    fn interrupt_handle(&mut self, dev: &mut Devices, address: u16, interrupt_type: InterruptType) -> bool {
        // Two internal NOPs
        dev.incr_cycle(1);
        dev.incr_cycle(1);

        // The two push operations
        self.push(&mut dev.bus, (self.registers.pc >> 8) as u8);
        dev.incr_cycle(1);

        let ie_register = dev.bus.get_ie_register();
        let it = interrupt_type as u8;

        if (ie_register & it) == 0 { // The interrupt was cancelled
            self.registers.pc = 0;
            return true
        }

        self.push(&mut dev.bus, self.registers.pc as u8);
        dev.incr_cycle(1);

        if (ie_register & it) == 0 { // The interrupt was cancelled
            self.registers.pc = 0;
            return true
        }

        // Final jump
        self.registers.pc = address;
        dev.incr_cycle(1);
        return false
    }

    pub(super) fn handle_interrupts(&mut self, dev: &mut Devices) {
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

            let cancelled = self.interrupt_handle(dev, address, interrupt_type);

            self.halted = false;
            self.int_master_enabled = false;

            if !cancelled {
                self.set_int_flags(dev, if_register & !it);
                return true
            } else {
                return false
            }
        }
        false
    }
}