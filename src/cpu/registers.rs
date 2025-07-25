use super::instruction::RegType;

pub struct CpuRegisters {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
}

impl CpuRegisters {
    pub fn new() -> CpuRegisters {
        CpuRegisters { a: 0x01, f: 0, b: 0, c: 0, d: 0, e: 0, h: 0, l: 0, pc: 0x100, sp: 0 }
    }

    pub fn read(&self, register: RegType) -> u16 {
        match register {
            RegType::NONE => 0,

            RegType::A => self.a as u16,
            RegType::F => self.f as u16,
            RegType::B => self.b as u16,
            RegType::C => self.c as u16,
            RegType::D => self.d as u16,
            RegType::E => self.e as u16,
            RegType::H => self.h as u16,
            RegType::L => self.l as u16,

            RegType::AF => (self.a as u16) << 8 | self.f as u16,
            RegType::BC => (self.b as u16) << 8 | self.c as u16,
            RegType::DE => (self.d as u16) << 8 | self.e as u16,
            RegType::HL => (self.h as u16) << 8 | self.l as u16,

            RegType::SP => self.sp as u16,
            RegType::PC => self.pc as u16,
        }
    }

    pub fn set(&mut self, register: RegType, value: u16) {
        match register {
            RegType::NONE => (),

            RegType::A => self.a = value as u8,
            RegType::F => self.f = value as u8,
            RegType::B => self.b = value as u8,
            RegType::C => self.c = value as u8,
            RegType::D => self.d = value as u8,
            RegType::E => self.e = value as u8,
            RegType::H => self.h = value as u8,
            RegType::L => self.l = value as u8,

            RegType::AF => {
                self.a = ((value & 0xFF00) >> 8) as u8;
                self.f = value as u8;
            },
            RegType::BC => {
                self.b = ((value & 0xFF00) >> 8) as u8;
                self.c = value as u8;
            },
            RegType::DE => {
                self.d = ((value & 0xFF00) >> 8) as u8;
                self.e = value as u8;
            },
            RegType::HL => {
                self.h = ((value & 0xFF00) >> 8) as u8;
                self.l = value as u8;
            },
            RegType::SP => self.sp = value,
            RegType::PC => self.pc = value,
        }
    }
}