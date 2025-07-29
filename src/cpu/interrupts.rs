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
    fn interrupt_handle(&mut self, bus: &mut Interconnect, address: u16) {
        self.push16(bus, self.registers.pc);
        self.registers.pc = address;
    }

    pub fn handle_interrupts(&mut self, bus: &mut Interconnect) {
        if self.interrupt_check(bus, 0x40, InterruptType::VBlank) {

        } else if self.interrupt_check(bus, 0x48, InterruptType::LcdStat) {

        } else if self.interrupt_check(bus, 0x50, InterruptType::Timer) {

        } else if self.interrupt_check(bus, 0x58, InterruptType::Serial) {

        } else if self.interrupt_check(bus, 0x60, InterruptType::Joypad) {
            
        }
    }

    fn interrupt_check(&mut self, bus: &mut Interconnect, address: u16, interrupt_type: InterruptType) -> bool {
        if (self.int_flags & interrupt_type.value()) != 0 && 
            (bus.get_ie_register() & interrupt_type.value()) != 0 {
            self.interrupt_handle(bus, address);
            self.int_flags |= !interrupt_type.value();
            self.halted = false;
            self.int_master_enabled = false;
            return true
        }
        false
    }
}