use std::{fs::File, io::{Read, Write}, path::PathBuf};

use crate::cart::{CartridgeInternals};

use super::CartridgeHeader;

const RAM_BANK_SIZE: usize = 0x2000;

pub struct MBC1 {
    rom_data: Vec<u8>,

    // MBC1-related data
    ram_enabled: bool,

    active_rom_bank: usize, // Offset of the ROM bank
    banking_mode: u8,

    active_ram_bank: usize, // Currently selected RAM bank
    ram_banks: [Option<Box<[u8; RAM_BANK_SIZE]>>; 16], // All RAM banks

    // for battery
    battery: bool,
    need_save: bool,
}

impl MBC1 {
    pub fn new(header: &CartridgeHeader, rom_data: Vec<u8>) -> MBC1 {
        let battery = header.cart_type == 3;
        let need_save = false;

        let ram_enabled = header.ram_size != 0;

        let mut ram_banks: [Option<Box<[u8; RAM_BANK_SIZE]>>; 16] = [const { None }; 16];
        if ram_enabled {
            // RAM Banks initialization
            for i in 0..16 {
                ram_banks[i] = match header.ram_size {
                    2 if i == 0 => Some(Box::new([0; 0x2000])),
                    3 if i < 4 => Some(Box::new([0; 0x2000])),
                    4 if i < 16 => Some(Box::new([0; 0x2000])),
                    5 if i < 8 => Some(Box::new([0; 0x2000])),
                    _ => None,
                }
            }
        }

        let active_ram_bank = 0;
        let active_rom_bank = 1; // ROM bank 1

        MBC1 {
            rom_data,
            ram_enabled,

            active_rom_bank,
            banking_mode: 0,

            active_ram_bank,
            ram_banks,

            battery,
            need_save,
        }
    }
}


impl CartridgeInternals for MBC1 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom_data[address as usize],

            0x4000..=0x7FFF => {
                let offset = self.active_rom_bank * 0x4000;
                self.rom_data[offset + (address as usize - 0x4000)]
            }

            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return 0xFF;
                }

                self.ram_banks[self.active_ram_bank]
                    .as_ref()
                    .unwrap()[address as usize - 0xA000]
            }
            _ => 0xFF,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            ..0x2000 => self.ram_enabled = (value & 0xF) == 0xA,
            0x2000..0x4000 => self.active_rom_bank = (value as usize & 0x1F).max(1),
            0x4000..0x6000 => self.active_ram_bank = value as usize & 0b11,
            0x6000..0x8000 => self.banking_mode = value & 1,
            0xA000..0xC000 if self.ram_enabled => {
                self.ram_banks[self.active_ram_bank]
                    .as_mut()                                
                    .unwrap()[address as usize - 0xA000] = value;
                if self.battery {
                    self.need_save = true;
                }
            }
            _ => (),
        }
    }

    fn need_save(&mut self) -> bool {
        let need_save = self.need_save;
        if need_save { self.need_save = false };
        need_save
    }

    fn save(&self, save_path: &PathBuf) {
        // Determine how many banks should be saved
        let bank_count = self.ram_banks.iter()
            .filter(|&bank| bank.is_some())
            .count();

        let mut buffer = Vec::with_capacity(bank_count * 0x2000);

        for i in 0..bank_count {
            if let Some(bank) = &self.ram_banks[i] {
                buffer.extend_from_slice(&**bank);
            } else {
                // If a bank is missing, pad with zeros
                buffer.extend_from_slice(&[0u8; 0x2000]);
            }
        }

        let mut file = File::create(save_path).expect("Failed to create save file");
        file.write_all(&buffer).expect("Failed to write save file");
    }

    fn load_save(&mut self, save_path: &PathBuf) {
        // If the save file doesn't exist 
        // it will be created on next frame anyway
        if let Ok(mut file) = File::open(save_path) {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            
            let bank_count = self.ram_banks.iter()
                .filter(|&bank| bank.is_some())
                .count();
            let expected_len = bank_count * 0x2000;

            if buffer.len() == expected_len {
                for i in 0..bank_count {
                    let start = i * 0x2000;
                    let end = start + 0x2000;
                    let slice = &buffer[start..end];

                    // Copy the slice into a boxed array
                    let mut arr = Box::new([0u8; 0x2000]);
                    arr.copy_from_slice(slice);
                    self.ram_banks[i] = Some(arr);
                }
            } else {
                panic!("Failed to load the save data: incorrect size")
            }
        }
    }
}