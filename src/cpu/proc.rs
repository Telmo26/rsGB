use super::{instruction::*, CPU};

use crate::{Interconnect, EmuContext};

impl CPU {
    pub fn execute(&mut self, bus: &mut Interconnect, ctx: &mut EmuContext, instruction: InType) { // -> impl FnMut(&mut CPU, &mut Interconnect, &mut EmuContext) {
        match instruction {
            InType::NOP => proc_nop(self, bus, ctx),
            InType::LD => proc_ld(self, bus, ctx),
            InType::LDH => proc_ldh(self, bus, ctx),
            InType::JP => proc_jp(self, bus, ctx),
            InType::JR => proc_jr(self, bus, ctx),
            InType::CALL => proc_call(self, bus, ctx),
            InType::RET => proc_ret(self, bus, ctx),
            InType::RETI => proc_reti(self, bus, ctx),
            InType::DI => proc_di(self, bus, ctx),
            InType::POP => proc_pop(self, bus, ctx),
            InType::PUSH => proc_push(self, bus, ctx),
            InType::XOR => proc_xor(self, bus, ctx),
            x => panic!("Instruction {x:?} not implemented")
        }
    }
}

fn proc_nop(_cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {}

fn proc_ld(cpu: &mut CPU, bus: &mut Interconnect, _ctx: &mut EmuContext) {
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

fn proc_ldh(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext) {
    if cpu.curr_inst.reg_1 == RegType::A {
        // Loading into register A
        cpu.registers.set(RegType::A, cpu.fetched_data);
    } else {
        // Loading A into a memory region
        bus.write(cpu.mem_dest, cpu.fetched_data as u8)
    }

    ctx.incr_cycle();
}


fn goto_addr(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext, address: u16, push_pc: bool) {
    if cpu.check_cond() {
        if push_pc {
            ctx.incr_cycle();
            ctx.incr_cycle();
            cpu.push16(bus, cpu.registers.pc);
        }
        cpu.registers.pc = address;
        ctx.incr_cycle();
    }
}

fn proc_jp(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext) {
    goto_addr(cpu, bus, ctx, cpu.fetched_data, false);
}

fn proc_jr(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext) {
    let rel = cpu.fetched_data as i8;
    let (addr, _) = cpu.registers.pc.overflowing_add_signed(rel as i16);
    goto_addr(cpu, bus, ctx, addr, false);
}

fn proc_call(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext) {
    goto_addr(cpu, bus, ctx, cpu.fetched_data, true);
}

fn proc_ret(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext) {
    if cpu.curr_inst.cond != CondType::NONE {
        ctx.incr_cycle();
    }

    if cpu.check_cond() {
        let low: u16 = cpu.pop(bus) as u16;
        ctx.incr_cycle();

        let high: u16 = cpu.pop(bus) as u16;
        ctx.incr_cycle();

        cpu.registers.pc = (high << 8) | low;

        ctx.incr_cycle();
    }

    goto_addr(cpu, bus, ctx, cpu.fetched_data, false);
}

fn proc_reti(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext) {
    cpu.en_master_interrupt = true;
    proc_ret(cpu, bus, ctx);
}

fn proc_di(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    cpu.en_master_interrupt = false;
}

fn proc_pop(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext) {
    let low = cpu.pop(bus) as u16;
    ctx.incr_cycle();

    let high = cpu.pop(bus) as u16;
    ctx.incr_cycle();

    let data = (high << 8) | low;

    if cpu.curr_inst.reg_1 == RegType::AF {
        cpu.registers.set(RegType::AF, data & 0xFFF0);
    } else {
        cpu.registers.set(RegType::AF, data);
    }
}

fn proc_push(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext) {
    let high = (cpu.registers.read(cpu.curr_inst.reg_1) >> 8) as u8;
    ctx.incr_cycle();
    cpu.push(bus, high);

    let low = cpu.registers.read(cpu.curr_inst.reg_1) as u8;
    ctx.incr_cycle();
    cpu.push(bus, low);

    ctx.incr_cycle();
}

fn proc_xor(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    cpu.registers.a ^= (cpu.fetched_data & 0xFF) as u8;
    let z_flag = if cpu.registers.a == 0 { 1 } else { 0 };
    cpu.set_flags(z_flag, 0, 0, 0)
}