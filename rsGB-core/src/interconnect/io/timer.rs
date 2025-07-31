use crate::{cpu::interrupts::InterruptType, interconnect::{io::IO, Interconnect}};

#[derive(Debug)]
pub struct Timer {
    div: u16,
    tima: u8,
    tma: u8,
    tac: u8,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            div: 0xAC00,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    pub fn tick(&mut self) -> Option<InterruptType> {
        let prev_div = self.div;

        self.div = self.div.wrapping_add(1);

        let timer_update: bool;

        match self.tac & 0b11 {
            0b00 => timer_update = ((prev_div & (1 << 9)) != 0) && ((self.div & (1 << 9)) == 0),
            0b01 => timer_update = ((prev_div & (1 << 3)) != 0) && ((self.div & (1 << 3)) == 0),
            0b10 => timer_update = ((prev_div & (1 << 5)) != 0) && ((self.div & (1 << 5)) == 0),
            0b11 => timer_update = ((prev_div & (1 << 7)) != 0) && ((self.div & (1 << 7)) == 0),
            _ => panic!() // This is mathematically impossible to reach
        }

        if timer_update && ((self.tac & (1 << 2)) != 0) {
            self.tima += 1;

            if self.tima == 0xFF {
                self.tima = self.tma;

                return Some(InterruptType::Timer)
            }
        }
        None
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => self.div = 0,
            0xFF05 => {
                self.tima = value;
                println!("Setting TIMA to {value}");
            } 
            0xFF06 => self.tma = value,
            0xFF07 => self.tac = value,
            _ => panic!()
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => panic!()
        }
    }
}
