use std::collections::HashMap;

use crate::{cpu::{CPU}, utils::VRAM};

pub struct DebugInfo<'dbg> {
    pub(crate) cpu: &'dbg CPU,
    pub(crate) vram: &'dbg VRAM,
    instruction_text: String,
}

impl<'dbg> DebugInfo<'dbg> {
    pub fn new(cpu: &'dbg CPU, vram: &'dbg VRAM) -> DebugInfo<'dbg> {
        DebugInfo {
            cpu,
            vram,
            instruction_text: String::new(),
        }
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
}