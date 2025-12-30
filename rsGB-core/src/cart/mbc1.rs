use std::{fs::File, io::{Read, Write}, path::PathBuf};

use crate::cart::{CartridgeInternals};

use super::CartridgeHeader;

const RAM_BANK_SIZE: usize = 0x2000;

pub struct MBC1 {
    rom_data: Vec<u8>,

    // MBC1-related data
    ram_enabled: bool,

    bank1: usize, // BANK 1 register
    bank2: usize, // BANK 2 register

    banking_mode: u8,

    ram_bank_nb: u8,
    ram_banks: Vec<[u8; RAM_BANK_SIZE]>, // All RAM banks

    // for battery
    battery: bool,
    need_save: bool,
}

impl MBC1 {
    pub fn new(header: &CartridgeHeader, rom_data: Vec<u8>) -> MBC1 {
        let battery = header.cart_type == 3;
        let need_save = false;

        println!("ROM Data: {:X}", rom_data.len());
       
        let ram_bank_nb: u8 = match header.ram_size {
            0x00 => 0,
            0x02 => 1,
            0x03 => 4,
            0x04 => 16,
            0x05 => 8,
            _ => unreachable!(),
        };


        println!("{} RAM banks in the cartridge", ram_bank_nb);

        let mut ram_banks = Vec::with_capacity(ram_bank_nb as usize);
        for _ in 0..ram_bank_nb {
            ram_banks.push([0; 0x2000]);
        }

        MBC1 {
            rom_data,
            ram_enabled: false,

            bank1: 1, // This is the ROM bank register: it cannot be 0
            bank2: 0,

            banking_mode: 0,

            ram_bank_nb,
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
                let offset = self.bank1 << 14;

                self.rom_data[offset + (address as usize - 0x4000)]
            }

            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return 0xFF;
                }

                let ram_bank = if self.banking_mode == 0 {
                    0
                } else {
                    self.bank2
                };
                self.ram_banks[ram_bank][address as usize % (1 << 12)]
            }
            _ => 0xFF,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            ..0x2000 => self.ram_enabled = (value & 0xF) == 0xA,
            0x2000..0x4000 => self.bank1 = (value as usize & 0x1F).max(1),
            0x4000..0x6000 => {
                if self.ram_bank_nb > 0 {
                    self.bank2 = (value as usize & 0b11).min(self.ram_bank_nb as usize - 1);
                }
            } 
            0x6000..0x8000 => self.banking_mode = value & 1,
            0xA000..0xC000 if self.ram_enabled => {
                let ram_bank = if self.banking_mode == 0 {
                    0
                } else {
                    self.bank2
                };
                self.ram_banks[ram_bank][address as usize % (1 << 12)] = value;

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
        return;
        // This way we only do one allocation
        let buffer: Vec<u8> = self.ram_banks.clone().into_iter()
            .flatten()
            .collect();

        let mut file = File::create(save_path).expect("Failed to create save file");
        file.write_all(&buffer).expect("Failed to write save file");
    }

    fn load_save(&mut self, save_path: &PathBuf) {
        // If the save file doesn't exist 
        // it will be created on next frame anyway
        if let Ok(mut file) = File::open(save_path) {
            let expected_len = self.ram_bank_nb as usize * 0x2000;

            let mut buffer = Vec::with_capacity(expected_len);
            file.read_to_end(&mut buffer).unwrap();
            
            if buffer.len() == expected_len {
                for i in 0..self.ram_bank_nb as usize {
                    let start = i * 0x2000;
                    let end = start + 0x2000;
                    let slice = &buffer[start..end];

                    // Copy the slice into a boxed array
                    self.ram_banks[i].copy_from_slice(slice);
                }
            } else {
                panic!("Failed to load the save data: incorrect size")
            }
        }
    }
}