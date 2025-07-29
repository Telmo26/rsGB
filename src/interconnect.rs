use crate::{
    cart::Cartridge,
};

mod ram;

use ram::*;

// 0x0000 - 0x3FFF : ROM Bank 0
// 0x4000 - 0x7FFF : ROM Bank 1 - Switchable
// 0x8000 - 0x97FF : CHR RAM
// 0x9800 - 0x9BFF : BG Map 1
// 0x9C00 - 0x9FFF : BG Map 2
// 0xA000 - 0xBFFF : Cartridge RAM
// 0xC000 - 0xCFFF : RAM Bank 0
// 0xD000 - 0xDFFF : RAM Bank 1-7 - switchable - Color only
// 0xE000 - 0xFDFF : Reserved - Echo RAM
// 0xFE00 - 0xFE9F : Object Attribute Memory
// 0xFEA0 - 0xFEFF : Reserved - Unusable
// 0xFF00 - 0xFF7F : I/O Registers
// 0xFF80 - 0xFFFE : Zero Page

pub struct Interconnect {
    cart: Option<Cartridge>,
    ram: RAM,
    ie_register: u8
}

impl Interconnect {
    pub fn new() -> Interconnect {
        Interconnect { cart: None, ram: RAM::new(), ie_register: 0 }
    }

    pub fn set_cart(&mut self, cart: Cartridge) {
        self.cart = Some(cart);
    }

    pub fn read(&self, address: u16) -> u8 {
        // ROM only for now
        match address {
            // ROM Data
            0x0000..0x8000 => self.cart.as_ref().unwrap().read(address),

            // Char/Map Data
            0x8000..0xA000 => 0, // panic!("Read at address {address:X} not implemented!"),

            // Cartridge RAM
            0xA000..0xC000 => self.cart.as_ref().unwrap().read(address),

            // WRAM (Working RAM)
            0xC000..0xE000 => self.ram.wram_read(address),

            // Reserved echo RAM
            0xE000..0xFE00 => 0,

            // OAM
            0xFE00..0xFEA0 => 0, // panic!("Read at address {address:X} not implemented!"),

            // Reserved - Unusable
            0xFEA0..0xFF00 => 0,

            // I/O Registers
            0xFF00..0xFF80 => 0, // panic!("Read at address {address:X} not implemented!"),

            // HRAM (High RAM) / Zero Page
            0xFF80..0xFFFF => self.ram.hram_read(address),

            // CPU Enable Register
            0xFFFF => self.ie_register,
        }
    }

    pub fn read16(&self, address: u16) -> u16 {
        let low: u16 = self.read(address) as u16;
        let high: u16 = self.read(address + 1) as u16;

        high << 8 | low
    }

    pub fn write(&mut self, address: u16, value: u8) {
        // ROM only for now
        match address {
            0x0000..0x8000 => self.cart.as_mut().unwrap().write(address, value),

           // Char/Map Data
            0x8000..0xA000 => println!("Write at address {address:X} not implemented!"),

            // Cartridge RAM
            0xA000..0xC000 => self.cart.as_mut().unwrap().write(address, value),

            // WRAM (Working RAM)
            0xC000..0xE000 => self.ram.wram_write(address, value),

            // Reserved echo RAM
            0xE000..0xFE00 => (),

            // OAM
            0xFE00..0xFEA0 => println!("Write at address {address:X} not implemented!"),

            // Reserved - Unusable
            0xFEA0..0xFF00 => (),

            // I/O Registers
            0xFF00..0xFF80 => println!("Write at address {address:X} not implemented!"),

            // HRAM (High RAM) / Zero Page
            0xFF80..0xFFFF => self.ram.hram_write(address, value),

            // CPU Enable Register
            0xFFFF => self.ie_register = value,
        }
    }

    pub fn write16(&mut self, address: u16, value: u16) {
        self.write(address, value as u8);
        self.write(address + 1, (value >> 8) as u8);
    }

    pub fn get_ie_register(&self) -> u8 {
        self.ie_register
    }
}

