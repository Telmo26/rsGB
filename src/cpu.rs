use core::hash;

struct CPURegisters {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
}

pub struct CPU {
    registers: CPURegisters,

    // Current fetch
    fetch_data: u16,
    mem_dest: u16,
    curr_opcode: u8,

    halted: bool,
    stepping: bool,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {}
    }

    pub fn step(&self) -> bool {
        println!("CPU not yet implemented.");
        false
    }
}