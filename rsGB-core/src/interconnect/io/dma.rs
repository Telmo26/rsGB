use std::{thread, time::Duration};

pub struct DMA {
    active: bool,
    byte: u8,
    value: u8,
    start_delay: u8,
}

impl DMA {
    pub fn new() -> DMA {
        DMA {
            active: false,
            byte: 0,
            value: 0,
            start_delay: 0
        }
    }

    pub fn tick(&mut self) -> Option<(u8, u8)> {
        if !self.active {
            return None
        }

        if self.start_delay > 0 {
            self.start_delay -= 1;
            return None
        }

        let prev_byte = self.byte;

        self.byte = self.byte.wrapping_add(1);
        self.active = self.byte < 0xA0;

        if !self.active {
            println!("DMA Done!");
            thread::sleep(Duration::from_secs(2));
        }

        Some((prev_byte, self.value))
    }

    pub fn start(&mut self, value: u8) {
        self.active = true;
        self.byte = 0;
        self.start_delay = 2;
        self.value = value;
    }

    pub fn transferring(&self) -> bool {
        self.active
    }
}