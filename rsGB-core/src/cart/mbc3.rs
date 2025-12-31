use std::{fs::File, io::{Read, Write}};

use crate::cart::{CartridgeInternals, header::CartridgeHeader};

mod rtc;
use rtc::RTC;

const RAM_BANK_SIZE: usize = 0x2000;

pub struct MBC3 {
    rom_data: Vec<u8>,
    active_rom_bank: usize,
    rom_bank_nb: usize,

    ram_rtc_enabled: bool,
    mapped_memory: MappedMemory,
    previous_latch: u8,

    ram_banks: Vec<[u8; RAM_BANK_SIZE]>,
    rtc: RTC,

    battery_present: bool,
    timer_present: bool,
    need_save: bool,
}

impl MBC3 {
    pub fn new(header: &CartridgeHeader, rom_data: Vec<u8>) -> MBC3 {
        let rom_bank_nb = rom_data.len() / 0x4000; // A ROM bank is 32 KiB
        
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

        let battery_present = header.cart_type == 0xF || header.cart_type == 0x10 || header.cart_type == 0x13;
        let timer_present = header.cart_type == 0x0F || header.cart_type == 0x10;

        MBC3 {
            rom_data,
            active_rom_bank: 1,
            rom_bank_nb,

            ram_rtc_enabled: false,
            mapped_memory: MappedMemory::RamBank(0),
            previous_latch: 0xFF,

            ram_banks,
            rtc: RTC::default(),

            battery_present,
            timer_present,
            need_save: false,
        }
    }
}

impl CartridgeInternals for MBC3 {
    fn read(&self, address: u16) -> u8 {
        match address {
            ..=0x3FFF => self.rom_data[address as usize],
            0x4000..=0x7FFF => {
                let bank = self.active_rom_bank % self.rom_bank_nb;
                let offset = (bank as usize) << 14;
                self.rom_data[offset | (address as usize & 0x3FFF)]
            }
            0xA000..=0xBFFF => {
                if self.ram_rtc_enabled {
                    match self.mapped_memory {
                        MappedMemory::RamBank(idx) => if idx < self.ram_banks.len() as u8 {
                            self.ram_banks[idx as usize][address as usize & 0x1FFF]
                        } else { 0xFF }
                        MappedMemory::RtcRegister(reg) => if self.timer_present {
                            self.rtc.read(reg)
                        } else { 0xFF }
                    }
                } else { 0xFF }
            }
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            ..=0x1FFF => self.ram_rtc_enabled = (value & 0x0F) == 0x0A,
            0x2000..=0x3FFF => self.active_rom_bank = (value & 0x7F).max(1) as usize,
            0x4000..=0x5FFF => match value {
                ..=0x07 => if value < self.ram_banks.len() as u8 {
                    self.mapped_memory = MappedMemory::RamBank(value);
                }
                0x08..=0x0C => if self.timer_present {
                    self.mapped_memory = MappedMemory::RtcRegister(value);
                }
                _ => {},
            }
            0x6000..=0x7FFF => {
                if self.previous_latch == 0 && (value == 1) {
                    self.rtc.latch();
                }
                self.previous_latch = value;
            }
            0xA000..=0xBFFF => if self.ram_rtc_enabled {
                match self.mapped_memory {
                    MappedMemory::RtcRegister(reg) => self.rtc.write(reg, value),
                    MappedMemory::RamBank(idx) => {
                        self.ram_banks[idx as usize][address as usize & 0x1FFF] = value;
                        
                        if self.battery_present {
                            self.need_save = true;
                        }
                    } 
                }
            }
            _ => unreachable!(),
        }
    }

    fn need_save(&mut self) -> bool {
        let need_save = self.need_save;
        if need_save { self.need_save = false };
        need_save
    }

    fn load_save(&mut self, save_path: &std::path::PathBuf) {
        // If the save file doesn't exist 
        // it will be created on next frame anyway
        if let Ok(mut file) = File::open(save_path) {
            let expected_len = self.ram_banks.len() * 0x2000;

            let mut buffer = Vec::with_capacity(expected_len);
            file.read_to_end(&mut buffer).unwrap();
            
            if buffer.len() == expected_len {
                for i in 0..self.ram_banks.len() {
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

        if self.timer_present { self.rtc.load(save_path); }
    }

    fn save(&self, save_path: &std::path::PathBuf) {
        // This way we only do one allocation
        let buffer: Vec<u8> = self.ram_banks.clone().into_iter()
            .flatten()
            .collect();

        let mut file = File::create(save_path).expect("Failed to create save file");
        file.write_all(&buffer).expect("Failed to write save file");
        
        if self.timer_present { self.rtc.save(save_path); }
    }
}

enum MappedMemory {
    RamBank(u8),
    RtcRegister(u8),
}

