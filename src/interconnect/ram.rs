pub struct RAM {
    wram: [u8; 0x2000],
    hram: [u8; 0x80],
}

impl RAM {
    pub fn new() -> RAM {
        RAM { 
            wram: [0; 0x2000],
            hram: [0; 0x80],
        }
    }

    pub fn wram_read(&self, address: u16) -> u8 {
        let index = (address - 0xC000) as usize;

        assert!(index < self.wram.len());

        self.wram[index]
    }

    pub fn wram_write(&mut self, address: u16, value: u8) {
        let index = (address - 0xC000) as usize;

        assert!(index < self.wram.len());

        self.wram[index] = value;
    }

    pub fn hram_read(&self, address: u16) -> u8 {
        let index = (address - 0xFF80) as usize;

        assert!(index < self.hram.len());

        self.hram[index]
    }

    pub fn hram_write(&mut self, address: u16, value: u8) {
        let index = (address - 0xFF80) as usize;

        assert!(index < self.hram.len());

        self.hram[index] = value;
    }
}