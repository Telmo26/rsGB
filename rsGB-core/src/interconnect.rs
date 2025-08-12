use std::{cell::RefCell, sync::{Arc, Mutex}};

use crate::{
    cart::Cartridge, cpu::AddrMode, GamepadState,
};

pub use crate::{
    cpu::interrupts::InterruptType
};

mod ram;
mod io;
mod oam;

use ram::*;
use io::*;
pub use oam::OAMEntry;

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
    pub(crate) vram: Arc<Mutex<[u8; 0x2000]>>,
    ram: RAM,
    oam_ram: [OAMEntry; 40],
    io: IO,
    ie_register: u8,
}

impl Interconnect {
    pub fn new(vram: Arc<Mutex<[u8; 0x2000]>>,gamepad_state: Arc<Mutex<GamepadState>>) -> Interconnect {
        Interconnect { 
            cart: None,
            vram,
            ram: RAM::new(),
            oam_ram: [OAMEntry::new(); 40],
            io: IO::new(gamepad_state),
            ie_register: 0,
        }
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
            0x8000..0xA000 => {
                let vram = self.vram.lock().unwrap();
                vram[(address - 0x8000) as usize]
            }

            // Cartridge RAM
            0xA000..0xC000 => self.cart.as_ref().unwrap().read(address),

            // WRAM (Working RAM)
            0xC000..0xE000 => self.ram.wram_read(address),

            // Reserved echo RAM
            0xE000..0xFE00 => 0,

            // OAM
            0xFE00..0xFEA0 => {
                if self.io.dma_transferring() {
                    0xFF
                } else {
                    let sprite_index = ((address - 0xFE00)/4) as usize;
                    let byte = (address % 4) as u8;
                    self.oam_ram[sprite_index].read(byte)
                } 
            },

            // Reserved - Unusable
            0xFEA0..0xFF00 => 0,

            // I/O Registers
            0xFF00..0xFF80 => self.io.read(address), // panic!("Read at address {address:X} not implemented!"),

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
            0x8000..0xA000 => {
                let mut vram = self.vram.lock().unwrap();
                vram[(address - 0x8000) as usize] = value;
            },

            // Cartridge RAM
            0xA000..0xC000 => self.cart.as_mut().unwrap().write(address, value),

            // WRAM (Working RAM)
            0xC000..0xE000 => self.ram.wram_write(address, value),

            // Reserved echo RAM
            0xE000..0xFE00 => (),

            // OAM
            0xFE00..0xFEA0 => {
                if self.io.dma_transferring() {
                    return
                } else {
                    let sprite_index = ((address - 0xFE00)/4) as usize;
                    let byte = (address % 4) as u8;
                    self.oam_ram[sprite_index].write(byte, value);
                }
            },

            // Reserved - Unusable
            0xFEA0..0xFF00 => (),

            // I/O Registers
            0xFF00..0xFF80 => self.io.write(address, value),

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

    pub fn request_interrupt(&mut self, interrupt: InterruptType) {
        self.io.request_interrupt(interrupt);
    }

    /// This function ticks all the devices that tick once
    /// per clock cycle, like the DMA.
    pub fn tick_t(&mut self) {
        self.io.tick_timer();
        self.io.tick_apu();
    }


    /// This function ticks all the devices that tick once
    /// per machine cycle, like the DMA.
    pub fn tick_m(&mut self) {
        if let Some((byte, val)) = self.io.tick_dma() {
            let source_addr = val as u16 * 0x100 + byte as u16;
            let value = self.read(source_addr);
            let address = 0xFE00 | (byte as u16);

            let sprite_index = byte as usize / 4;
            let byte_offset = byte as u8 % 4;
            self.oam_ram[sprite_index].write(byte_offset, value);
        }
    }

    pub fn lcd_bg_colors(&self) -> &[u32; 4] {
        &self.io.lcd.bg_colors
    }

    pub fn lcd_sp1_colors(&self) -> &[u32; 4] {
        &self.io.lcd.sp1_colors
    }

    pub fn lcd_sp2_colors(&self) -> &[u32; 4] {
        &self.io.lcd.sp2_colors
    }

    pub fn oam_sprite(&self, index: u8) -> OAMEntry {
        assert!(index < 40);

        self.oam_ram[index as usize]
    }

    pub fn need_save(&self) -> bool {
        self.cart.as_ref().unwrap().need_save()
    }

    pub fn save(&self, save_path: &str) {
        self.cart.as_ref().unwrap().save(save_path);
    }

    pub fn load(&mut self, save_path: &str) {
        self.cart.as_mut().unwrap().load_save(save_path);
    }
}

