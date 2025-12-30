use std::{fs::File, io::{Read, Write}};

use crate::cart::{CartridgeInternals, header::CartridgeHeader};

pub struct MBC2 {
    rom_data: Vec<u8>,
    rom_bank_nb: u8,

    internal_ram: [u8; 512],
    ram_enabled: bool,

    has_battery: bool,
    need_save: bool,
}

impl MBC2 {
    pub fn new(header: &CartridgeHeader, rom_data: Vec<u8>) -> MBC2 {
        MBC2 {
            rom_data,
            rom_bank_nb: 1,

            internal_ram: [0; 512],
            ram_enabled: false,

            has_battery: header.cart_type == 0x06,
            need_save: false,
        }
    }
}

impl CartridgeInternals for MBC2 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom_data[address as usize],
            0x4000..=0x7FFF => self.rom_data[(self.rom_bank_nb as usize) << 14 | (address as usize & 0x3FFF)],
            0xA000..=0xBFFF => if self.ram_enabled { 
                self.internal_ram[(address as usize) & 0x1FF] & 0xF 
            } else { 0xFF },
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            ..=0x3FFF => {
                if address & (1 << 8) == 0 {
                    self.ram_enabled = (value & 0xF) == 0xA;
                } else {
                    self.rom_bank_nb = (value & 0xF).max(1);
                }
            }
            0x4000..=0x7FFF => {},
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    self.internal_ram[(address as usize) & 0x1FF] = value & 0xF;
                    if self.has_battery { self.need_save = true }
                }
            }
            _ => unreachable!()
        }
    }

    fn need_save(&mut self) -> bool {
        let need_save = self.need_save;
        if need_save { self.need_save = false };
        need_save
    }

    fn save(&self, save_path: &std::path::PathBuf) {
        let mut file = File::create(save_path).expect("Failed to create save file");
        file.write_all(&self.internal_ram).expect("Failed to write save file");
    }

    fn load_save(&mut self, save_path: &std::path::PathBuf) {
        if let Ok(mut file) = File::open(save_path) {
            let expected_len = self.internal_ram.len();

            let mut buffer = Vec::with_capacity(expected_len);
            file.read_to_end(&mut buffer).unwrap();
            
            if buffer.len() == expected_len {
                self.internal_ram.copy_from_slice(&buffer);
            } else {
                panic!("Failed to load the save data: incorrect size")
            }
        }
    }
}