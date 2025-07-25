use super::{instruction::{AddrMode, CondType, RegType, InType}, Cpu};

use crate::{interconnect::Interconnect, EmuContext};

pub fn cpu_proc(instruction: InType) -> impl FnMut(&mut Cpu, &mut Interconnect, &mut EmuContext) {
    match instruction {
        InType::NOP => proc_nop,
        InType::LD => proc_ld,
        InType::LDH => proc_ldh,
        InType::JP => proc_jp,
        InType::DI => proc_di,
        InType::XOR => proc_xor,
        x => panic!("Instruction {x:?} not implemented")
    }
}

fn proc_nop(_cpu: &mut Cpu, _bus: &mut Interconnect, _ctx: &mut EmuContext) {}

fn proc_ld(cpu: &mut Cpu, bus: &mut Interconnect, ctx: &mut EmuContext) {
    if cpu.curr_inst.mode == AddrMode::HL_SP {
        // Check if overflow from bit 3
        let hflag: bool = ((cpu.registers.read(cpu.curr_inst.reg_2) as u8 & 0xF) + 
            (cpu.fetched_data as u8 & 0xF)) >= 0x10;
        
        // Check if overflow from bit 7
        let (_, cflag) = (cpu.registers.read(cpu.curr_inst.reg_2) as u8)
            .overflowing_add(cpu.fetched_data as u8);

        cpu.set_flags(0, 0, hflag as u8, cflag as u8);
        cpu.registers.set(cpu.curr_inst.reg_1,
            cpu.registers.read(cpu.curr_inst.reg_2) + cpu.fetched_data);
    } else if cpu.dest_is_mem {
        if cpu.curr_inst.reg_2 >= RegType::AF {
            // If 16-bit register
            bus.write16(cpu.mem_dest, cpu.fetched_data);
        } else {
            bus.write(cpu.mem_dest, cpu.fetched_data as u8);
        }
    } else {
        cpu.registers.set(cpu.curr_inst.reg_1, cpu.fetched_data);
    }
}

fn proc_ldh(cpu: &mut Cpu, bus: &mut Interconnect, ctx: &mut EmuContext) {
    if cpu.curr_inst.reg_1 == RegType::A {
        // Loading into register A
        cpu.registers.set(RegType::A, cpu.fetched_data);
    } else {
        // Loading A into a memory region
        bus.write(cpu.mem_dest, cpu.fetched_data as u8)
    }

    ctx.incr_cycle();
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

fn proc_jp(cpu: &mut Cpu, _bus: &mut Interconnect, ctx: &mut EmuContext) {
    if check_cond(cpu) {
        cpu.registers.pc = cpu.fetched_data;
        ctx.incr_cycle();
    }
}

fn proc_di(cpu: &mut Cpu, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    cpu.en_master_interrupt = false;
}

fn proc_xor(cpu: &mut Cpu, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    cpu.registers.a ^= (cpu.fetched_data & 0xFF) as u8;
    let z_flag = if cpu.registers.a == 0 { 1 } else { 0 };
    cpu.set_flags(z_flag, 0, 0, 0)
}