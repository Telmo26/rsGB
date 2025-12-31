use crate::{cpu::interrupts::InterruptType};

#[derive(Debug, Default)]
pub struct Timer {
    pub(crate) div: u16,
    tima: u8,
    tma: u8,
    tac: u8,

    previous_result: bool,
    tima_overflow: bool,
    tima_overflow_counter: u8,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            div: 0xABCF,
            ..Timer::default()
        }
    }

    pub fn tick(&mut self) -> Option<InterruptType> {
        self.div = self.div.wrapping_add(1);

        if self.tima_overflow {
            self.tima_overflow_counter += 1;
            if self.tima_overflow_counter == 4 {
                self.tima = self.tma;
                self.tima_overflow = false;
                self.tima_overflow_counter = 0;
                return Some(InterruptType::Timer);
            }
        }

        let timer_enable = (self.tac & (1 << 2)) != 0;

        let result = timer_enable & self.selected_bit();

        if !self.tima_overflow && self.previous_result && !result { 
            // Falling edge and no overflow in the last 4 cycles
            let (new_tima, did_overflow) = self.tima.overflowing_add(1);
            self.tima = new_tima;

            if did_overflow {
                self.tima_overflow = did_overflow;
                self.tima_overflow_counter = 0;
            }
        }

        self.previous_result = result;
        None
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => {
                // write to DIV resets system counter to 0
                let timer_enable = (self.tac & (1 << 2)) != 0;
                self.div = 0;
                let new_result = timer_enable && self.selected_bit();

                // If write produced a falling edge and we are NOT in the special overflow suppression,
                // perform the same increment logic as a falling edge.
                if !self.tima_overflow && self.previous_result && !new_result {
                    let (new_tima, did_overflow) = self.tima.overflowing_add(1);
                    self.tima = new_tima;
                    if did_overflow {
                        self.tima_overflow = true;
                        self.tima_overflow_counter = 0;
                    }
                }
                self.previous_result = new_result;
            },
            0xFF05 => {
                self.tima = value;
                if self.tima_overflow {
                    // Bypass the overflow trigger
                    self.tima_overflow = false;
                    self.tima_overflow_counter = 0;
                }
            } 
            0xFF06 => self.tma = value,
            0xFF07 => {
                // Changing TAC can immediately generate a tick (falling edge) — emulate by checking old/new
                self.tac = value | 0b11111000;

                let new_timer_enable = (self.tac & (1 << 2)) != 0;
                let new_result = new_timer_enable && self.selected_bit();

                // If the change caused a falling edge (old true -> new false) and no overflow pending, fire tick
                if !self.tima_overflow && self.previous_result && !new_result {
                    let (new_tima, did_overflow) = self.tima.overflowing_add(1);
                    self.tima = new_tima;
                    if did_overflow {
                        self.tima_overflow = true;
                        self.tima_overflow_counter = 0;
                    }
                }
                self.previous_result = new_result;
            },
            _ => panic!()
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | 0b11111000,
            _ => panic!()
        }
    }

    fn selected_bit(&self) -> bool {
        match self.tac & 0b11 {
            0b00 => (self.div & (1 << 9)) != 0,
            0b01 => (self.div & (1 << 3)) != 0,
            0b10 => (self.div & (1 << 5)) != 0,
            0b11 => (self.div & (1 << 7)) != 0,
            _ => unreachable!()
        }
    }
}
