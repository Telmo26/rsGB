use super::CPU;

use crate::Interconnect;

impl CPU {
    pub fn push(&mut self, bus: &mut Interconnect, value: u8) {
        self.registers.sp -= 1;
        bus.write(self.registers.sp, value);
    }

    pub fn push16(&mut self, bus: &mut Interconnect, value: u16) {
        self.push(bus, (value >> 8) as u8);
        self.push(bus, value as u8);
    }

    pub fn pop(&mut self, bus: &mut Interconnect) -> u8 {
        let val = bus.read(self.registers.sp);
        self.registers.sp += 1;
        val
    }

    pub fn pop16(&mut self, bus: &mut Interconnect) -> u16 {
        let low = self.pop(bus) as u16;
        let high = self.pop(bus) as u16;
        high << 8 | low
    }
}