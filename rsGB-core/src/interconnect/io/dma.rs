pub struct DMA {
    active: bool,
    byte: u8,
    value: u8,
    start_delay: u8,
    restarted: bool,
}

impl DMA {
    pub fn new() -> DMA {
        DMA {
            active: false,
            byte: 0,
            value: 0,
            start_delay: 0,
            restarted: false,
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

        if self.restarted {
            self.restarted = false;
            self.byte = 0; 
            return None;
        }

        if self.byte < 0xA0 {
            let result = Some((self.byte, self.value));
            self.byte += 1;

            if self.byte == 0xA0 {
                self.active = false; 
            }
            
            return result
        }

        self.active = false;
        None
    }

    pub fn start(&mut self, value: u8) {       
        if self.transferring() {
            self.restarted = true;
            self.start_delay = 1;
        } else {
            self.start_delay = 2;
            self.byte = 0; 
        }

        self.active = true;       
        self.value = value;
    }

    pub fn transferring(&self) -> bool {
        (self.active && self.start_delay == 0) || self.restarted
    }
}