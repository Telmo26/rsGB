use crate::{cpu::registers::CpuRegisters, utils::VRAM};

pub struct DebugInfo<'dbg> {
    pub(crate) cpu_registers: &'dbg CpuRegisters,
    pub(crate) vram: &'dbg VRAM,
}

impl<'dbg> DebugInfo<'dbg> {
    /// This function always returns the 512 tiles from the VRAM
    pub fn get_tiles(&self) -> &[[u8; 16]] {
        self.vram.as_chunks::<16>().0
    }
}