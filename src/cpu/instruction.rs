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

impl RegType {
    pub fn is_16bit(&self) -> bool {
        *self >= RegType::AF
    }
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
    inst[0x03] = Some(Instruction { in_type: INC, mode: R, reg_1: BC, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x04] = Some(Instruction { in_type: INC, mode: R, reg_1: B, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x05] = Some(Instruction { in_type: DEC, mode: R, reg_1: B, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x06] = Some(Instruction { in_type: LD, mode: R_D8, reg_1: B, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x07] = Some(Instruction { in_type: RLCA, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });

    inst[0x08] = Some(Instruction { in_type: LD, mode: A16_R, reg_1: NONE, reg_2: SP, cond: CondType::NONE, param: 0 });
    inst[0x09] = Some(Instruction { in_type: ADD, mode: R_R, reg_1: HL, reg_2: BC, cond: CondType::NONE, param: 0 });
    inst[0x0A] = Some(Instruction { in_type: LD, mode: R_MR, reg_1: A, reg_2: BC, cond: CondType::NONE, param: 0 });
    inst[0x0B] = Some(Instruction { in_type: DEC, mode: R, reg_1: BC, reg_2: NONE, cond: CondType::NONE, param: 0 });    
    inst[0x0C] = Some(Instruction { in_type: INC, mode: R, reg_1: C, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x0D] = Some(Instruction { in_type: DEC, mode: R, reg_1: C, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x0E] = Some(Instruction { in_type: LD, mode: R_D8, reg_1: C, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x0F] = Some(Instruction { in_type: RRCA, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });

    // --- 0x10 - 0x1F ---
    inst[0x10] = Some(Instruction { in_type: STOP, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x11] = Some(Instruction { in_type: LD, mode: R_D16, reg_1: DE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x12] = Some(Instruction { in_type: LD, mode: MR_R, reg_1: DE, reg_2: A, cond: CondType::NONE, param: 0 });
    inst[0x13] = Some(Instruction { in_type: INC, mode: R, reg_1: DE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x14] = Some(Instruction { in_type: INC, mode: R, reg_1: D, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x15] = Some(Instruction { in_type: DEC, mode: R, reg_1: D, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x16] = Some(Instruction { in_type: LD, mode: R_D8, reg_1: D, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x17] = Some(Instruction { in_type: RLA, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });

    inst[0x18] = Some(Instruction { in_type: JR, mode: D8, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x19] = Some(Instruction { in_type: ADD, mode: R_R, reg_1: HL, reg_2: DE, cond: CondType::NONE, param: 0 });
    inst[0x1A] = Some(Instruction { in_type: LD, mode: R_MR, reg_1: A, reg_2: DE, cond: CondType::NONE, param: 0 });
    inst[0x1B] = Some(Instruction { in_type: DEC, mode: R, reg_1: DE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x1C] = Some(Instruction { in_type: INC, mode: R, reg_1: E, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x1D] = Some(Instruction { in_type: DEC, mode: R, reg_1: E, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x1E] = Some(Instruction { in_type: LD, mode: R_D8, reg_1: E, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x1F] = Some(Instruction { in_type: RRA, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });

    // --- 0x20 - 0x2F ---
    inst[0x20] = Some(Instruction { in_type: JR, mode: D8, reg_1: NONE, reg_2: NONE, cond: CondType::NZ, param: 0 });
    inst[0x21] = Some(Instruction { in_type: LD, mode: R_D16, reg_1: HL, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x22] = Some(Instruction { in_type: LD, mode: HLI_R, reg_1: NONE, reg_2: A, cond: CondType::NONE, param: 0 });
    inst[0x23] = Some(Instruction { in_type: INC, mode: R, reg_1: HL, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x24] = Some(Instruction { in_type: INC, mode: R, reg_1: H, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x25] = Some(Instruction { in_type: DEC, mode: R, reg_1: H, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x26] = Some(Instruction { in_type: LD, mode: R_D8, reg_1: H, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x27] = Some(Instruction { in_type: DAA, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });

    inst[0x28] = Some(Instruction { in_type: JR, mode: D8, reg_1: NONE, reg_2: NONE, cond: CondType::Z, param: 0 });
    inst[0x29] = Some(Instruction { in_type: ADD, mode: R_R, reg_1: HL, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x2A] = Some(Instruction { in_type: LD, mode: R_HLI, reg_1: A, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x2B] = Some(Instruction { in_type: DEC, mode: R, reg_1: HL, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x2C] = Some(Instruction { in_type: INC, mode: R, reg_1: L, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x2D] = Some(Instruction { in_type: DEC, mode: R, reg_1: L, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x2E] = Some(Instruction { in_type: LD, mode: R_D8, reg_1: L, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x2F] = Some(Instruction { in_type: CPL, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });

    // --- 0x30 - 0x3F ---
    inst[0x30] = Some(Instruction { in_type: JR, mode: D8, reg_1: NONE, reg_2: NONE, cond: CondType::NC, param: 0 });
    inst[0x31] = Some(Instruction { in_type: LD, mode: R_D16, reg_1: SP, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x32] = Some(Instruction { in_type: LD, mode: HLD_R, reg_1: HL, reg_2: A, cond: CondType::NONE, param: 0 });
    inst[0x33] = Some(Instruction { in_type: INC, mode: R, reg_1: SP, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x34] = Some(Instruction { in_type: INC, mode: MR, reg_1: HL, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x35] = Some(Instruction { in_type: DEC, mode: MR, reg_1: HL, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x36] = Some(Instruction { in_type: LD, mode: MR_D8, reg_1: HL, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x37] = Some(Instruction { in_type: SCF, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });

    inst[0x38] = Some(Instruction { in_type: JR, mode: D8, reg_1: NONE, reg_2: NONE, cond: CondType::C, param: 0 });
    inst[0x39] = Some(Instruction { in_type: ADD, mode: R_R, reg_1: HL, reg_2: SP, cond: CondType::NONE, param: 0 });
    inst[0x3A] = Some(Instruction { in_type: LD, mode: R_HLD, reg_1: A, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x3B] = Some(Instruction { in_type: DEC, mode: R, reg_1: SP, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x3C] = Some(Instruction { in_type: INC, mode: R, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x3D] = Some(Instruction { in_type: INC, mode: R, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x3E] = Some(Instruction { in_type: LD, mode: R_D8, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0x3F] = Some(Instruction { in_type: CCF, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });

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
    inst[0x76] = Some(Instruction { in_type: HALT, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
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
    inst[0x80] = Some(Instruction { in_type: ADD, mode: R_R, reg_1: A, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0x81] = Some(Instruction { in_type: ADD, mode: R_R, reg_1: A, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0x82] = Some(Instruction { in_type: ADD, mode: R_R, reg_1: A, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0x83] = Some(Instruction { in_type: ADD, mode: R_R, reg_1: A, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0x84] = Some(Instruction { in_type: ADD, mode: R_R, reg_1: A, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0x85] = Some(Instruction { in_type: ADD, mode: R_R, reg_1: A, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0x86] = Some(Instruction { in_type: ADD, mode: R_MR, reg_1: A, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x87] = Some(Instruction { in_type: ADD, mode: R_R, reg_1: A, reg_2: A, cond: CondType::NONE, param: 0 });

    inst[0x88] = Some(Instruction { in_type: ADC, mode: R_R, reg_1: A, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0x89] = Some(Instruction { in_type: ADC, mode: R_R, reg_1: A, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0x8A] = Some(Instruction { in_type: ADC, mode: R_R, reg_1: A, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0x8B] = Some(Instruction { in_type: ADC, mode: R_R, reg_1: A, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0x8C] = Some(Instruction { in_type: ADC, mode: R_R, reg_1: A, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0x8D] = Some(Instruction { in_type: ADC, mode: R_R, reg_1: A, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0x8E] = Some(Instruction { in_type: ADC, mode: R_MR, reg_1: A, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x8F] = Some(Instruction { in_type: ADC, mode: R_R, reg_1: A, reg_2: A, cond: CondType::NONE, param: 0 });
    
    // --- 0x90 - 0x9F ---
    inst[0x90] = Some(Instruction { in_type: SUB, mode: R_R, reg_1: A, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0x91] = Some(Instruction { in_type: SUB, mode: R_R, reg_1: A, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0x92] = Some(Instruction { in_type: SUB, mode: R_R, reg_1: A, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0x93] = Some(Instruction { in_type: SUB, mode: R_R, reg_1: A, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0x94] = Some(Instruction { in_type: SUB, mode: R_R, reg_1: A, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0x95] = Some(Instruction { in_type: SUB, mode: R_R, reg_1: A, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0x96] = Some(Instruction { in_type: SUB, mode: R_MR, reg_1: A, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x97] = Some(Instruction { in_type: SUB, mode: R_R, reg_1: A, reg_2: A, cond: CondType::NONE, param: 0 });

    inst[0x98] = Some(Instruction { in_type: SBC, mode: R_R, reg_1: A, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0x99] = Some(Instruction { in_type: SBC, mode: R_R, reg_1: A, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0x9A] = Some(Instruction { in_type: SBC, mode: R_R, reg_1: A, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0x9B] = Some(Instruction { in_type: SBC, mode: R_R, reg_1: A, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0x9C] = Some(Instruction { in_type: SBC, mode: R_R, reg_1: A, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0x9D] = Some(Instruction { in_type: SBC, mode: R_R, reg_1: A, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0x9E] = Some(Instruction { in_type: SBC, mode: R_MR, reg_1: A, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0x9F] = Some(Instruction { in_type: SBC, mode: R_R, reg_1: A, reg_2: A, cond: CondType::NONE, param: 0 });
    
    // --- 0xA0 - 0xAF ---
    inst[0xA0] = Some(Instruction { in_type: AND, mode: R_R, reg_1: A, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0xA1] = Some(Instruction { in_type: AND, mode: R_R, reg_1: A, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0xA2] = Some(Instruction { in_type: AND, mode: R_R, reg_1: A, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0xA3] = Some(Instruction { in_type: AND, mode: R_R, reg_1: A, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0xA4] = Some(Instruction { in_type: AND, mode: R_R, reg_1: A, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0xA5] = Some(Instruction { in_type: AND, mode: R_R, reg_1: A, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0xA6] = Some(Instruction { in_type: AND, mode: R_MR, reg_1: A, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0xA7] = Some(Instruction { in_type: AND, mode: R_R, reg_1: A, reg_2: A, cond: CondType::NONE, param: 0 });

    inst[0xA8] = Some(Instruction { in_type: XOR, mode: R_R, reg_1: A, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0xA9] = Some(Instruction { in_type: XOR, mode: R_R, reg_1: A, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0xAA] = Some(Instruction { in_type: XOR, mode: R_R, reg_1: A, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0xAB] = Some(Instruction { in_type: XOR, mode: R_R, reg_1: A, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0xAC] = Some(Instruction { in_type: XOR, mode: R_R, reg_1: A, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0xAD] = Some(Instruction { in_type: XOR, mode: R_R, reg_1: A, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0xAE] = Some(Instruction { in_type: XOR, mode: R_MR, reg_1: A, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0xAF] = Some(Instruction { in_type: XOR, mode: R_R, reg_1: A, reg_2: A, cond: CondType::NONE, param: 0 });

    // --- 0xB0 - 0xBF ---
    inst[0xB0] = Some(Instruction { in_type: OR, mode: R_R, reg_1: A, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0xB1] = Some(Instruction { in_type: OR, mode: R_R, reg_1: A, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0xB2] = Some(Instruction { in_type: OR, mode: R_R, reg_1: A, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0xB3] = Some(Instruction { in_type: OR, mode: R_R, reg_1: A, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0xB4] = Some(Instruction { in_type: OR, mode: R_R, reg_1: A, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0xB5] = Some(Instruction { in_type: OR, mode: R_R, reg_1: A, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0xB6] = Some(Instruction { in_type: OR, mode: R_MR, reg_1: A, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0xB7] = Some(Instruction { in_type: OR, mode: R_R, reg_1: A, reg_2: A, cond: CondType::NONE, param: 0 });

    inst[0xB8] = Some(Instruction { in_type: CP, mode: R_R, reg_1: A, reg_2: B, cond: CondType::NONE, param: 0 });
    inst[0xB9] = Some(Instruction { in_type: CP, mode: R_R, reg_1: A, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0xBA] = Some(Instruction { in_type: CP, mode: R_R, reg_1: A, reg_2: D, cond: CondType::NONE, param: 0 });
    inst[0xBB] = Some(Instruction { in_type: CP, mode: R_R, reg_1: A, reg_2: E, cond: CondType::NONE, param: 0 });
    inst[0xBC] = Some(Instruction { in_type: CP, mode: R_R, reg_1: A, reg_2: H, cond: CondType::NONE, param: 0 });
    inst[0xBD] = Some(Instruction { in_type: CP, mode: R_R, reg_1: A, reg_2: L, cond: CondType::NONE, param: 0 });
    inst[0xBE] = Some(Instruction { in_type: CP, mode: R_MR, reg_1: A, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0xBF] = Some(Instruction { in_type: CP, mode: R_R, reg_1: A, reg_2: A, cond: CondType::NONE, param: 0 });

    // --- 0xC0 - 0xCF ---
    inst[0xC0] = Some(Instruction { in_type: RET, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NZ, param: 0 });
    inst[0xC1] = Some(Instruction { in_type: POP, mode: IMP, reg_1: BC, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xC2] = Some(Instruction { in_type: JP, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::NZ, param: 0 });
    inst[0xC3] = Some(Instruction { in_type: JP, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xC4] = Some(Instruction { in_type: CALL, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::NZ, param: 0 });
    inst[0xC5] = Some(Instruction { in_type: PUSH, mode: IMP, reg_1: BC, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xC6] = Some(Instruction { in_type: ADD, mode: R_D8, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xC7] = Some(Instruction { in_type: RST, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0x00 });

    inst[0xC8] = Some(Instruction { in_type: RET, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::Z, param: 0 });
    inst[0xC9] = Some(Instruction { in_type: RET, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xCA] = Some(Instruction { in_type: JP, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::Z, param: 0 });
    inst[0xCB] = Some(Instruction { in_type: CB, mode: D8, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xCC] = Some(Instruction { in_type: CALL, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::Z, param: 0 });
    inst[0xCD] = Some(Instruction { in_type: CALL, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xCF] = Some(Instruction { in_type: RST, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0x08 });
    
    // --- 0xD0 - 0xDF ---
    inst[0xD0] = Some(Instruction { in_type: RET, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NC, param: 0 });
    inst[0xD1] = Some(Instruction { in_type: POP, mode: IMP, reg_1: DE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xD2] = Some(Instruction { in_type: JP, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::NC, param: 0 });
    inst[0xD4] = Some(Instruction { in_type: CALL, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::NC, param: 0 });
    inst[0xD5] = Some(Instruction { in_type: PUSH, mode: IMP, reg_1: DE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xD6] = Some(Instruction { in_type: SUB, mode: R_D8, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xD7] = Some(Instruction { in_type: RST, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0x10 });

    inst[0xD8] = Some(Instruction { in_type: RET, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::C, param: 0 });
    inst[0xD9] = Some(Instruction { in_type: RETI, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xDA] = Some(Instruction { in_type: JP, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::C, param: 0 });
    inst[0xDC] = Some(Instruction { in_type: CALL, mode: D16, reg_1: NONE, reg_2: NONE, cond: CondType::C, param: 0 });
    inst[0xDE] = Some(Instruction { in_type: SBC, mode: R_D8, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xDF] = Some(Instruction { in_type: RST, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0x18 });

    // --- 0xE0 - 0xEF ---
    inst[0xE0] = Some(Instruction { in_type: LDH, mode: A8_R, reg_1: NONE, reg_2: A, cond: CondType::NONE, param: 0 });
    inst[0xE1] = Some(Instruction { in_type: POP, mode: IMP, reg_1: HL, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xE2] = Some(Instruction { in_type: LDH, mode: MR_R, reg_1: A, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0xE5] = Some(Instruction { in_type: PUSH, mode: IMP, reg_1: HL, reg_2: NONE, cond: CondType::NONE, param: 0 }); 
    inst[0xE6] = Some(Instruction { in_type: AND, mode: R_D8, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });   
    inst[0xE7] = Some(Instruction { in_type: RST, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0x20 });

    inst[0xE8] = Some(Instruction { in_type: ADD, mode: R_D8, reg_1: SP, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xE9] = Some(Instruction { in_type: JP, mode: MR, reg_1: HL, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xEA] = Some(Instruction { in_type: LD, mode: A16_R, reg_1: NONE, reg_2: A, cond: CondType::NONE, param: 0 });
    inst[0xEE] = Some(Instruction { in_type: XOR, mode: R_D8, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xEF] = Some(Instruction { in_type: RST, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0x28 });

    // --- 0xF0 - 0xFF ---
    inst[0xF0] = Some(Instruction { in_type: LDH, mode: R_A8, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xF1] = Some(Instruction { in_type: POP, mode: IMP, reg_1: AF, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xF2] = Some(Instruction { in_type: LDH, mode: R_MR, reg_1: A, reg_2: C, cond: CondType::NONE, param: 0 });
    inst[0xF3] = Some(Instruction { in_type: DI, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xF5] = Some(Instruction { in_type: PUSH, mode: IMP, reg_1: AF, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xF6] = Some(Instruction { in_type: OR, mode: R_D8, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });    
    inst[0xF7] = Some(Instruction { in_type: RST, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0x30 });

    inst[0xF8] = Some(Instruction { in_type: LD, mode: HL_SP, reg_1: HL, reg_2: SP, cond: CondType::NONE, param: 0 });
    inst[0xF9] = Some(Instruction { in_type: LD, mode: R_R, reg_1: SP, reg_2: HL, cond: CondType::NONE, param: 0 });
    inst[0xFA] = Some(Instruction { in_type: LD, mode: R_A16, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xFB] = Some(Instruction { in_type: EI, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xFE] = Some(Instruction { in_type: CP, mode: R_D8, reg_1: A, reg_2: NONE, cond: CondType::NONE, param: 0 });
    inst[0xFF] = Some(Instruction { in_type: RST, mode: IMP, reg_1: NONE, reg_2: NONE, cond: CondType::NONE, param: 0x38 });

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