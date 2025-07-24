use super::{instruction::InType, Cpu};

use crate::{cpu::instruction::CondType, EmuContext};

pub fn cpu_proc(instruction: InType) -> impl FnMut(&mut Cpu, &mut EmuContext) {
    match instruction {
        InType::NONE => proc_none,
        InType::NOP => proc_nop,
        InType::LD => proc_ld,
        InType::JP => proc_jp,
        InType::DI => proc_di,
        InType::XOR => proc_xor,
        x => panic!("Instruction {x:?} not implemented")
    }
}

fn proc_none(_cpu: &mut Cpu, _ctx: &mut EmuContext) {
    panic!("INVALID INSTRUCTION")
}

fn proc_nop(_cpu: &mut Cpu, _ctx: &mut EmuContext) {}

fn proc_ld(cpu: &mut Cpu, ctx: &mut EmuContext) {
    // TODO...
}

fn check_cond(cpu: &mut Cpu) -> bool {
    let z = cpu.z_flag();
    let c = cpu.c_flag();

    match cpu.curr_inst.cond {
        CondType::NONE => true,
        CondType::Z => z,
        CondType::NZ => !z,
        CondType::C => c,
        CondType::NC => !c
    }
}

fn proc_jp(cpu: &mut Cpu, ctx: &mut EmuContext) {
    if check_cond(cpu) {
        cpu.registers.pc = cpu.fetched_data;
        ctx.incr_cycle(1);
    }
}

fn proc_di(cpu: &mut Cpu, _ctx: &mut EmuContext) {
    cpu.en_master_interrupt = false;
}

fn proc_xor(cpu: &mut Cpu, _ctx: &mut EmuContext) {
    cpu.registers.a ^= (cpu.fetched_data & 0xFF) as u8;
    let z_flag = if cpu.registers.a == 0 { 1 } else { 0 };
    cpu.set_flags(z_flag, 0, 0, 0)
}