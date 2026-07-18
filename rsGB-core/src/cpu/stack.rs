use super::CPU;

use crate::Peripherals;

impl CPU {
    pub(crate) fn push(&mut self, dev: &mut impl Peripherals, value: u8) {
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        dev.write8(self.registers.sp, value);
    }

    // pub(crate) fn push16(&mut self, dev: &mut impl Peripherals, value: u16) {
    //     self.push(dev, (value >> 8) as u8);
    //     self.push(dev, value as u8);
    // }

    pub(crate) fn pop(&mut self, dev: &mut impl Peripherals) -> u8 {
        let val = dev.read8(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        val
    }
}