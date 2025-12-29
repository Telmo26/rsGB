use std::collections::HashMap;

use crate::{cart::Cartridge, cpu::CPU, utils::VRAM};

pub struct DebugInfo<'dbg> {
    cpu: &'dbg CPU,

    vram_updated: bool,
    vram: &'dbg VRAM,

    cartridge: &'dbg Cartridge,
    instruction_text: String,
}

impl<'dbg> DebugInfo<'dbg> {
    pub fn new(cpu: &'dbg CPU, vram_updated: bool, vram: &'dbg VRAM, cartridge: &'dbg Cartridge) -> DebugInfo<'dbg> {
        DebugInfo {
            cpu,
            vram_updated,
            vram,
            cartridge,
            instruction_text: String::new(),
        }
    } 

    pub fn vram_updated(&self) -> bool {
        self.vram_updated
    }

    /// This function always returns the 512 tiles from the VRAM
    pub fn get_tiles(&self) -> &[[u8; 16]] {
        self.vram.as_chunks::<16>().0
    }
    
    pub fn current_instruction(&mut self) -> &str {
        let cpu = self.cpu;
        self.instruction_text = cpu.curr_inst.to_str(&cpu);
        &self.instruction_text
    }

    /// This function returns a HashMap with each register value accessible by name
    pub fn registers(&self) -> HashMap<&str, u16> {
        let mut reg = HashMap::with_capacity(10);

        reg.insert("a", self.cpu.registers.a as u16);
        reg.insert("f", self.cpu.registers.f as u16);
        reg.insert("b", self.cpu.registers.b as u16);
        reg.insert("c", self.cpu.registers.c as u16);
        reg.insert("d", self.cpu.registers.d as u16);
        reg.insert("e", self.cpu.registers.e as u16);
        reg.insert("h", self.cpu.registers.h as u16);
        reg.insert("l", self.cpu.registers.l as u16); 
        reg.insert("pc", self.cpu.registers.pc);
        reg.insert("sp", self.cpu.registers.sp);  

        reg
    }

    pub fn game_name(&self) -> &str {
        &self.cartridge.header.title
    }

    pub fn game_license(&self) -> &str {
        self.cartridge.header.get_lic_name()
    }

    pub fn game_cartridge_type(&self) -> &str {
        self.cartridge.header.get_cart_type()
    }

    pub fn game_supports_sgb(&self) -> bool {
        self.cartridge.header.sgb_flag == 0x03
    }

    pub fn game_type(&self) -> &str {
        match self.cartridge.header.cgb_flag {
            0..128 => "DMG",
            0x80 => "DMG with CGB enhancements",
            0xC0 => "CGB only",
            _ => "Unknown"
        }
    }

    // Returns the size of the ROM in KiB
    pub fn game_rom_size(&self) -> u16 {
        32 << self.cartridge.header.rom_size
    }

    // Returns the size of the RAM in KiB
    pub fn game_ram_size(&self) -> u8 {
        match self.cartridge.header.ram_size {
            0x02 => 8,
            0x03 => 32,
            0x04 => 128,
            0x05 => 64,
            _ => 0,
        }
    }
}