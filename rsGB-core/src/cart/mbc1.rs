use std::{fs::File, io::{Read, Write}, path::PathBuf};

use crate::cart::{CartridgeInternals};

use super::CartridgeHeader;

const RAM_BANK_SIZE: usize = 0x2000;

const LOGO_OFFSET: usize = 0x104;
const LOGO_LEN: usize = 48;

const HEADER_CHECKSUM_OFFSET: usize = 0x014D;

const MULTICART_SLOT_SIZE: usize = 0x40000;

const NINTENDO_LOGO: [u8; LOGO_LEN] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
    0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
    0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
    0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

pub struct MBC1 {
    rom_data: Vec<u8>,
    rom_bank_nb: u8,
    is_multicart: bool,

    // MBC1-related data
    ram_enabled: bool,

    bank1: usize, // BANK 1 register
    bank2: usize, // BANK 2 register

    banking_mode: u8,

    ram_banks: Vec<[u8; RAM_BANK_SIZE]>,
    ram_bank_nb: u8,

    // for battery
    battery: bool,
    need_save: bool,
}

impl MBC1 {
    pub fn new(header: &CartridgeHeader, rom_data: Vec<u8>) -> MBC1 {
        let battery = header.cart_type == 3;
        let need_save = false;

        let rom_bank_nb = (rom_data.len() / 0x4000) as u8; // A rom_data bank is 16 KiB
       
        let is_multicart = detect_multicart(&rom_data);

        let ram_bank_nb: u8 = match header.ram_size {
            0x00 => 0,
            0x02 => 1,
            0x03 => 4,
            0x04 => 16,
            0x05 => 8,
            _ => unreachable!(),
        };

        let mut ram_banks = Vec::with_capacity(ram_bank_nb as usize);
        for _ in 0..ram_bank_nb {
            ram_banks.push([0; 0x2000]);
        }

        MBC1 {
            rom_data,
            rom_bank_nb,
            is_multicart,

            ram_enabled: false,

            bank1: 1, // This is the rom_data bank register: it cannot be 0
            bank2: 0,

            banking_mode: 0,

            ram_banks,
            ram_bank_nb,
            
            battery,
            need_save,
        }
    }
}


impl CartridgeInternals for MBC1 {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => {
                let mut bank = if self.banking_mode == 0 {
                    0
                } else if self.is_multicart {
                    self.bank2 << 4
                } else {
                    self.bank2 << 5
                };                

                bank %= self.rom_bank_nb as usize;

                self.rom_data[bank << 14 | address as usize & 0x3FFF]
            }

            0x4000..=0x7FFF => {
                let mut bank = if self.is_multicart {
                    (self.bank2 << 4) | (self.bank1 & 0xF)
                } else {
                    (self.bank2 << 5) | self.bank1
                };
                if !self.is_multicart { bank = bank.max(1); }
                
                bank %= self.rom_bank_nb as usize;

                self.rom_data[bank << 14 | address as usize & 0x3FFF]
            }

            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return 0xFF;
                }

                let ram_bank = if self.banking_mode == 0 {
                    0
                } else {
                    self.bank2.min(self.ram_bank_nb as usize - 1)
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
            0x4000..0x6000 => self.bank2 = value as usize & 0b11,
            0x6000..0x8000 => self.banking_mode = value & 1,
            0xA000..0xC000 if self.ram_enabled && self.ram_bank_nb > 0 => {
                let ram_bank = if self.banking_mode == 0 {
                    0
                } else {
                    self.bank2.min(self.ram_bank_nb as usize - 1)
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

fn detect_multicart(rom_data: &Vec<u8>) -> bool {
    if rom_data.len() < MULTICART_SLOT_SIZE * 2 {
        return false;
    }

    let mut headers_found = 0;

    let slots = rom_data.len() / MULTICART_SLOT_SIZE;

    for slot in 0..slots {
        let base = slot * MULTICART_SLOT_SIZE;

        if valid_gb_header(rom_data, base) {
            headers_found += 1;

            // One valid header is normal, two means multicart
            if headers_found >= 2 {
                return true;
            }
        }
    }

    false
}

fn valid_gb_header(rom_data: &[u8], base: usize) -> bool {
    if base + HEADER_CHECKSUM_OFFSET >= rom_data.len() {
        return false;
    }

    // Nintendo logo check
    if rom_data[base + LOGO_OFFSET .. base + LOGO_OFFSET + LOGO_LEN] != NINTENDO_LOGO {
        return false;
    }

    // Header checksum check
    let mut checksum: u8 = 0;
    for i in 0x0134..=0x014C {
        checksum = checksum.wrapping_sub(rom_data[base + i]).wrapping_sub(1);
    }

    checksum == rom_data[base + HEADER_CHECKSUM_OFFSET]
}