use crate::{NO_IMPL};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AddrMode {
    IMP,
    R_D16,
    R_R,
    MR_R,
    R,
    R_D8,
    R_MR,
    R_HLI,
    R_HLD,
    HLI_R,
    HLD_R,
    R_A8,
    A8_R,
    HL_SP,
    D16,
    D8,
    D16_R,
    MR_D8,
    MR,
    A16_R,
    R_A16,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CondType {
    NONE,
    NZ,
    Z,
    NC,
    C,
}

pub static INSTRUCTIONS: [Option<Instruction>; 0x100] = {
    use InType::*;
    use AddrMode::*;
    use RegType::*;
    let mut inst = [const { None }; 0x100];

    // --- 0x00 - 0x0F ---
    inst[0x00] = Some(Instruction { in_type: NOP, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x01] = Some(Instruction { in_type: LD, mode: R_D16, reg_1: BC, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x02] = Some(Instruction { in_type: LD, mode: MR_R, reg_1: BC, reg_2: A, cond: CondType::NONE, param: 0 });
    inst[0x05] = Some(Instruction { in_type: DEC, mode: R, reg_1: B, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x06] = Some(Instruction { in_type: LD, mode: R_D8, reg_1: B, reg_2: NONE, cond: CondType::NONE, param: 0 });
    
    inst[0x08] = Some(Instruction { in_type: LD, mode: A16_R, reg_1: NONE, reg_2: SP, cond: CondType::NONE, param: 0 });
    inst[0x0A] = Some(Instruction { in_type: LD, mode: R_MR, reg_1: A, reg_2: BC, cond: CondType::NONE, param: 0 });
    inst[0x0E] = Some(Instruction { in_type: LD, mode: R_D8, reg_1: C, reg_2: NONE, cond: CondType::NONE, param: 0 });

    // --- 0x10 - 0x1F ---
    inst[0x11] = Some(Instruction { in_type: LD, mode: R_D16, reg_1: DE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x12] = Some(Instruction { in_type: LD, mode: MR_R, reg_1: DE, reg_2: A, cond: CondType::NONE, param: 0 });
    inst[0x16] = Some(Instruction { in_type: LD, mode: R_D8, reg_1: D, reg_2: NONE, cond: CondType::NONE, param: 0 });
    
    inst[0x18] = Some(Instruction { in_type: JR, mode: D8, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x1A] = Some(Instruction { in_type: LD, mode: R_MR, reg_1: A, reg_2: DE, cond: CondType::NONE, param: 0 });
    inst[0x1E] = Some(Instruction { in_type: LD, mode: R_D8, reg_1: E, reg_2: NONE, cond: CondType::NONE, param: 0 });

    // --- 0x20 - 0x2F ---
    inst[0x20] = Some(Instruction { in_type: JR, mode: D8, reg_1: NONE, reg_2: NONE, cond: CondType::NZ, param: 0 });
    inst[0x21] = Some(Instruction { in_type: LD, mode: R_D16, reg_1: HL, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x22] = Some(Instruction { in_type: LD, mode: HLI_R, reg_1: NONE, reg_2: A, cond: CondType::NONE, param: 0 });
    inst[0x26] = Some(Instruction { in_type: LD, mode: R_D8, reg_1: H, reg_2: NONE, cond: CondType::NONE, param: 0 });
    
    inst[0x28] = Some(Instruction { in_type: JR, mode: D8, reg_1: NONE, reg_2: NONE, cond: CondType::Z, param: 0 });
    inst[0x2A] = Some(Instruction { in_type: LD, mode: R_MR, reg_1: A, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x2E] = Some(Instruction { in_type: LD, mode: R_D8, reg_1: L, reg_2: NONE, cond: CondType::NONE, param: 0 });

    // --- 0x30 - 0x3F ---
    inst[0x30] = Some(Instruction { in_type: JR, mode: D8, reg_1: NONE, reg_2: NONE, cond: CondType::NC, param: 0 });
    inst[0x31] = Some(Instruction { in_type: LD, mode: R_D16, reg_1: SP, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x32] = Some(Instruction { in_type: LD, mode: HLD_R, reg_1: HL, reg_2: A, cond: CondType::NONE, param: 0 });
    inst[0x36] = Some(Instruction { in_type: LD, mode: MR_D8, reg_1: HL, reg_2: NONE, cond: CondType::NONE, param: 0 });
    
    inst[0x38] = Some(Instruction { in_type: JR, mode: D8, reg_1: NONE, reg_2: NONE, cond: CondType::C, param: 0 });
    inst[0x3A] = Some(Instruction { in_type: LD, mode: R_MR, reg_1: A, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x3E] = Some(Instruction { in_type: LD, mode: R_D8, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });

    // --- 0x40 - 0x4F ---
    inst[0x40] = Some(Instruction { in_type: LD, mode: R_R, reg_1: B, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0x41] = Some(Instruction { in_type: LD, mode: R_R, reg_1: B, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0x42] = Some(Instruction { in_type: LD, mode: R_R, reg_1: B, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0x43] = Some(Instruction { in_type: LD, mode: R_R, reg_1: B, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0x44] = Some(Instruction { in_type: LD, mode: R_R, reg_1: B, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0x45] = Some(Instruction { in_type: LD, mode: R_R, reg_1: B, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0x46] = Some(Instruction { in_type: LD, mode: R_MR, reg_1: B, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x47] = Some(Instruction { in_type: LD, mode: R_R, reg_1: B, reg_2: A, cond: CondType::NONE, param: 0 });

    inst[0x48] = Some(Instruction { in_type: LD, mode: R_R, reg_1: C, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0x49] = Some(Instruction { in_type: LD, mode: R_R, reg_1: C, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0x4A] = Some(Instruction { in_type: LD, mode: R_R, reg_1: C, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0x4B] = Some(Instruction { in_type: LD, mode: R_R, reg_1: C, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0x4C] = Some(Instruction { in_type: LD, mode: R_R, reg_1: C, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0x4D] = Some(Instruction { in_type: LD, mode: R_R, reg_1: C, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0x4E] = Some(Instruction { in_type: LD, mode: R_MR, reg_1: C, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x4F] = Some(Instruction { in_type: LD, mode: R_R, reg_1: C, reg_2: A, cond: CondType::NONE, param: 0 });

    // --- 0x50 - 0x5F ---
    inst[0x50] = Some(Instruction { in_type: LD, mode: R_R, reg_1: D, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0x51] = Some(Instruction { in_type: LD, mode: R_R, reg_1: D, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0x52] = Some(Instruction { in_type: LD, mode: R_R, reg_1: D, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0x53] = Some(Instruction { in_type: LD, mode: R_R, reg_1: D, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0x54] = Some(Instruction { in_type: LD, mode: R_R, reg_1: D, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0x55] = Some(Instruction { in_type: LD, mode: R_R, reg_1: D, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0x56] = Some(Instruction { in_type: LD, mode: R_MR, reg_1: D, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x57] = Some(Instruction { in_type: LD, mode: R_R, reg_1: D, reg_2: A, cond: CondType::NONE, param: 0 });

    inst[0x58] = Some(Instruction { in_type: LD, mode: R_R, reg_1: E, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0x59] = Some(Instruction { in_type: LD, mode: R_R, reg_1: E, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0x5A] = Some(Instruction { in_type: LD, mode: R_R, reg_1: E, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0x5B] = Some(Instruction { in_type: LD, mode: R_R, reg_1: E, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0x5C] = Some(Instruction { in_type: LD, mode: R_R, reg_1: E, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0x5D] = Some(Instruction { in_type: LD, mode: R_R, reg_1: E, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0x5E] = Some(Instruction { in_type: LD, mode: R_MR, reg_1: E, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x5F] = Some(Instruction { in_type: LD, mode: R_R, reg_1: E, reg_2: A, cond: CondType::NONE, param: 0 });

    // --- 0x60 - 0x6F ---
    inst[0x60] = Some(Instruction { in_type: LD, mode: R_R, reg_1: H, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0x61] = Some(Instruction { in_type: LD, mode: R_R, reg_1: H, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0x62] = Some(Instruction { in_type: LD, mode: R_R, reg_1: H, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0x63] = Some(Instruction { in_type: LD, mode: R_R, reg_1: H, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0x64] = Some(Instruction { in_type: LD, mode: R_R, reg_1: H, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0x65] = Some(Instruction { in_type: LD, mode: R_R, reg_1: H, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0x66] = Some(Instruction { in_type: LD, mode: R_MR, reg_1: H, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x67] = Some(Instruction { in_type: LD, mode: R_R, reg_1: H, reg_2: A, cond: CondType::NONE, param: 0 });

    inst[0x68] = Some(Instruction { in_type: LD, mode: R_R, reg_1: L, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0x69] = Some(Instruction { in_type: LD, mode: R_R, reg_1: L, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0x6A] = Some(Instruction { in_type: LD, mode: R_R, reg_1: L, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0x6B] = Some(Instruction { in_type: LD, mode: R_R, reg_1: L, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0x6C] = Some(Instruction { in_type: LD, mode: R_R, reg_1: L, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0x6D] = Some(Instruction { in_type: LD, mode: R_R, reg_1: L, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0x6E] = Some(Instruction { in_type: LD, mode: R_MR, reg_1: L, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x6F] = Some(Instruction { in_type: LD, mode: R_R, reg_1: L, reg_2: A, cond: CondType::NONE, param: 0 });

    // --- 0x70 - 0x7F ---
    inst[0x70] = Some(Instruction { in_type: LD, mode: MR_R, reg_1: HL, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0x71] = Some(Instruction { in_type: LD, mode: MR_R, reg_1: HL, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0x72] = Some(Instruction { in_type: LD, mode: MR_R, reg_1: HL, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0x73] = Some(Instruction { in_type: LD, mode: MR_R, reg_1: HL, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0x74] = Some(Instruction { in_type: LD, mode: MR_R, reg_1: HL, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0x75] = Some(Instruction { in_type: LD, mode: MR_R, reg_1: HL, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0x76] = None; // HALT instruction
    inst[0x77] = Some(Instruction { in_type: LD, mode: MR_R, reg_1: HL, reg_2: A, cond: CondType::NONE, param: 0 });

    inst[0x78] = Some(Instruction { in_type: LD, mode: R_R, reg_1: A, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0x79] = Some(Instruction { in_type: LD, mode: R_R, reg_1: A, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0x7A] = Some(Instruction { in_type: LD, mode: R_R, reg_1: A, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0x7B] = Some(Instruction { in_type: LD, mode: R_R, reg_1: A, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0x7C] = Some(Instruction { in_type: LD, mode: R_R, reg_1: A, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0x7D] = Some(Instruction { in_type: LD, mode: R_R, reg_1: A, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0x7E] = Some(Instruction { in_type: LD, mode: R_MR, reg_1: A, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x7F] = Some(Instruction { in_type: LD, mode: R_R, reg_1: A, reg_2: A, cond: CondType::NONE, param: 0 });

    // --- 0x80 - 0x8F ---

    // --- 0x90 - 0x9F ---

    // --- 0xA0 - 0xAF ---
    inst[0xAF] = Some(Instruction { in_type: XOR, mode: R, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });

    // --- 0xB0 - 0xBF ---

    // --- 0xC0 - 0xCF ---
    inst[0xC0] = Some(Instruction { in_type: RET, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NZ, param: 0 });
    inst[0xC1] = Some(Instruction { in_type: POP, mode: IMP, reg_1: BC, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xC2] = Some(Instruction { in_type: JP, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::NZ, param: 0 });
    inst[0xC3] = Some(Instruction { in_type: JP, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xC4] = Some(Instruction { in_type: CALL, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::NZ, param: 0 });
    inst[0xC5] = Some(Instruction { in_type: PUSH, mode: IMP, reg_1: BC, reg_2: NONE, cond: CondType::NONE, param: 0 });

    inst[0xC8] = Some(Instruction { in_type: RET, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::Z, param: 0 });
    inst[0xC9] = Some(Instruction { in_type: RET, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xCA] = Some(Instruction { in_type: JP, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::Z, param: 0 });
    inst[0xCC] = Some(Instruction { in_type: CALL, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::Z, param: 0 });
    inst[0xCD] = Some(Instruction { in_type: CALL, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    
    // --- 0xD0 - 0xDF ---
    inst[0xD0] = Some(Instruction { in_type: RET, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NC, param: 0 });
    inst[0xD1] = Some(Instruction { in_type: POP, mode: IMP, reg_1: DE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xD2] = Some(Instruction { in_type: JP, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::NC, param: 0 });
    inst[0xD4] = Some(Instruction { in_type: CALL, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::NC, param: 0 });
    inst[0xD5] = Some(Instruction { in_type: PUSH, mode: IMP, reg_1: DE, reg_2: NONE, cond: CondType::NONE, param: 0 });

    inst[0xD8] = Some(Instruction { in_type: RET, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::C, param: 0 });
    inst[0xD9] = Some(Instruction { in_type: RETI, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xDA] = Some(Instruction { in_type: JP, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::C, param: 0 });
    inst[0xDC] = Some(Instruction { in_type: CALL, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::C, param: 0 });

    // --- 0xE0 - 0xEF ---
    inst[0xE0] = Some(Instruction { in_type: LDH, mode: A8_R, reg_1: NONE, reg_2: A, cond: CondType::NONE, param: 0 });
    inst[0xE1] = Some(Instruction { in_type: POP, mode: IMP, reg_1: HL, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xE2] = Some(Instruction { in_type: LDH, mode: MR_R, reg_1: A, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0xE5] = Some(Instruction { in_type: PUSH, mode: IMP, reg_1: HL, reg_2: NONE, cond: CondType::NONE, param: 0 });    
    
    inst[0xE9] = Some(Instruction { in_type: JP, mode: MR, reg_1: HL, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xEA] = Some(Instruction { in_type: LD, mode: A16_R, reg_1: NONE, reg_2: A, cond: CondType::NONE, param: 0 });

    // --- 0xF0 - 0xFF ---
    inst[0xF0] = Some(Instruction { in_type: LDH, mode: R_A8, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xF1] = Some(Instruction { in_type: POP, mode: IMP, reg_1: AF, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xF2] = Some(Instruction { in_type: LDH, mode: R_MR, reg_1: A, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0xF3] = Some(Instruction { in_type: DI, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xF5] = Some(Instruction { in_type: PUSH, mode: IMP, reg_1: AF, reg_2: NONE, cond: CondType::NONE, param: 0 });    
    
    inst[0xFA] = Some(Instruction { in_type: LD, mode: R_A16, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });

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