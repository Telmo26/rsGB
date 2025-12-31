#![allow(non_contiguous_range_endpoints)]

use std::{error::Error, fs, path::PathBuf};

mod header;
use header::CartridgeHeader;

mod rom;
mod mbc1;
mod mbc2;
mod mbc3;

use self::{
    rom::ROM, mbc1::MBC1, mbc2::MBC2, mbc3::MBC3
};


trait CartridgeInternals {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn need_save(&mut self) -> bool;
    fn save(&self, save_path: &PathBuf);
    fn load_save(&mut self, save_path: &PathBuf);
}

pub struct Cartridge {
    _rom_size: u32,
    pub(crate) header: CartridgeHeader,
    cart_internals: Box<dyn CartridgeInternals + Send>,
}

impl Cartridge {
    pub fn load(path: &PathBuf) -> Result<Cartridge, Box<dyn Error>> {
        let rom_data = fs::read(path)?;
        let rom_size = (rom_data.len() * 8) as u32;

        let header = CartridgeHeader::from_bytes(&rom_data)?;

        let cart_internals: Box<dyn CartridgeInternals + Send> = match header.cart_type {
            0 => Box::new(ROM::new(rom_data)),
            0x1..0x4 => Box::new(MBC1::new(&header, rom_data)),
            0x5..0x7 => Box::new(MBC2::new(&header, rom_data)),
            0x0F..=0x13 => Box::new(MBC3::new(&header, rom_data)),
            _ => panic!("Incompatible MBC detected: {:X}", header.cart_type)
        };

        Ok(Cartridge {
            _rom_size: rom_size,
            header,
            cart_internals,
        })
    }

    pub fn read(&self, address: u16) -> u8 {
        self.cart_internals.read(address)
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.cart_internals.write(address, value);
    }

    pub fn save(&self, save_path: &PathBuf) {
        self.cart_internals.save(save_path);
    }

    pub fn load_save(&mut self, save_path: &PathBuf) {
        self.cart_internals.load_save(save_path);
    }

    pub fn need_save(&mut self) -> bool {
        self.cart_internals.need_save()
    }
}
