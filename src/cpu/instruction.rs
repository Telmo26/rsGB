use crate::{NO_IMPL};

#[derive(Clone, Copy, Debug)]
pub enum AddrMode {
    IMP,
    RD16,
    RR,
    MRR,
    R,
    RD8,
    RMR,
    RHLI,
    RHLD,
    HLIR,
    HLDR,
    RA8,
    A8R,
    HLSPR,
    D16,
    D8,
    D16R,
    MRD8,
    MR,
    A16R,
    RA16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RegType {
    NONE,
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

#[derive(Clone, Copy, Debug)]
pub enum InType {
    NONE,
    NOP,
    LD,
    INC,
    DEC,
    RLCA,
    ADD,
    RRCA,
    STOP,
    RLA,
    JR,
    RRA,
    DAA,
    CPL,
    SCF,
    CCF,
    HALT,
    ADC,
    SUB,
    SBC,
    AND,
    XOR,
    OR,
    CP,
    POP,
    JP,
    PUSH,
    RET,
    CB,
    CALL,
    RETI,
    LDH,
    JPHL,
    DI,
    EI,
    RST,
    ERR,
    // CB INSTRUCTIONS...
    RLC,
    RRC,
    RL,
    RR,
    SLA,
    SRA,
    SWAP,
    SRL,
    BIT,
    RES,
    SET,
}

#[derive(Clone, Copy, Debug)]
pub enum CondType {
    NONE,
    NZ,
    Z,
    NC,
    C,
}

pub static INSTRUCTIONS: [Option<Instruction>; 0x100] = {
    let mut inst = [const { None }; 0x100];

    inst[0x00] = Some(Instruction { in_type: InType::NOP, mode: AddrMode::IMP, reg_1: RegType::NONE, reg_2: RegType::NONE, cond: CondType::NONE, param: 0});

    inst[0x05] = Some(Instruction { in_type: InType::DEC, mode: AddrMode::R, reg_1: RegType::B, reg_2: RegType::NONE, cond: CondType::NONE, param: 0});

    inst[0x0E] = Some(Instruction { in_type: InType::LD, mode: AddrMode::RD8, reg_1: RegType::C, reg_2: RegType::NONE, cond: CondType::NONE, param: 0});
    
    inst[0xAF] = Some(Instruction { in_type: InType::XOR, mode: AddrMode::R, reg_1: RegType::A, reg_2: RegType::NONE, cond: CondType::NONE, param: 0});

    inst[0xC3] = Some(Instruction { in_type: InType::JP, mode: AddrMode::D16, reg_1: RegType::NONE, reg_2: RegType::NONE, cond: CondType::NONE, param: 0});

    inst[0xF3] = Some(Instruction { in_type: InType::DI, mode: AddrMode::IMP, reg_1: RegType::NONE, reg_2: RegType::NONE, cond: CondType::NONE, param: 0});
    
    inst
};

#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    pub in_type: InType,
    pub mode: AddrMode,
    pub reg_1: RegType,
    pub reg_2: RegType,
    pub cond: CondType,
    pub param: u8,
}

impl Instruction {
    pub fn from_opcode(opcode: u8) -> Instruction {
        INSTRUCTIONS[opcode as usize]
            .expect(&format!("Opcode {opcode:X} not implemented!"))
    }
}