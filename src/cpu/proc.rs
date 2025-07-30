use super::{instruction::*, CPU};

use crate::{utils::*, EmuContext, Interconnect};

impl CPU {
    pub fn execute(&mut self, bus: &mut Interconnect, ctx: &mut EmuContext, instruction: InType) { // -> impl FnMut(&mut CPU, &mut Interconnect, &mut EmuContext) {
        match instruction {
            InType::NOP => proc_nop(self, bus, ctx),
            InType::LD => proc_ld(self, bus, ctx),
            InType::LDH => proc_ldh(self, bus, ctx),
            InType::JP => proc_jp(self, bus, ctx),
            InType::JR => proc_jr(self, bus, ctx),
            InType::CALL => proc_call(self, bus, ctx),
            InType::RST => proc_rst(self, bus, ctx),
            InType::RET => proc_ret(self, bus, ctx),
            InType::RETI => proc_reti(self, bus, ctx),
            InType::DI => proc_di(self, bus, ctx),
            InType::POP => proc_pop(self, bus, ctx),
            InType::PUSH => proc_push(self, bus, ctx),
            InType::INC => proc_inc(self, bus, ctx),
            InType::DEC => proc_dec(self, bus, ctx),
            InType::ADD => proc_add(self, bus, ctx),
            InType::ADC => proc_adc(self, bus, ctx),
            InType::SUB => proc_sub(self, bus, ctx),
            InType::SBC => proc_sbc(self, bus, ctx),
            InType::AND => proc_and(self, bus, ctx),
            InType::XOR => proc_xor(self, bus, ctx),
            InType::OR => proc_or(self, bus, ctx),
            InType::CP => proc_cp(self, bus, ctx),
            InType::CB => proc_cb(self, bus, ctx),
            InType::RLCA => proc_rlca(self, bus, ctx),
            InType::RRCA => proc_rrca(self, bus, ctx),
            InType::RLA => proc_rla(self, bus, ctx),
            InType::RRA => proc_rra(self, bus, ctx),
            InType::STOP => proc_stop(self, bus, ctx),
            InType::DAA => proc_daa(self, bus, ctx),
            InType::CPL => proc_cpl(self, bus, ctx),
            InType::SCF => proc_scf(self, bus, ctx),
            InType::CCF => proc_ccf(self, bus, ctx),
            InType::HALT => proc_halt(self, bus, ctx),
            InType::EI => proc_ei(self, bus, ctx),
            x => panic!("Instruction {x:?} not implemented")
        }
    }
}

/* These are local helper functions and lookup tables */
const REGISTER_LOOKUP: [RegType; 8] = [RegType::B, RegType::C, RegType::D, RegType::E, RegType::H, RegType::L, RegType::HL, RegType::A];

fn decode_register(register: u8) -> Option<RegType>{
    let reg = register as usize;
    if reg >= REGISTER_LOOKUP.len() {
        None
    } else {
        Some(REGISTER_LOOKUP[reg])
    }
}

/* These are the processing functions */

fn proc_nop(_cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {}

fn proc_ld(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext) {
    if cpu.curr_inst.mode == AddrMode::HL_SP {
        // Check if overflow from bit 3
        let hflag: bool = ((cpu.registers.read(cpu.curr_inst.reg_2) as u8 & 0xF) + 
            (cpu.fetched_data as u8 & 0xF)) >= 0x10;
        
        // Check if overflow from bit 7
        let (_, cflag) = (cpu.registers.read(cpu.curr_inst.reg_2) as u8)
            .overflowing_add(cpu.fetched_data as u8);

        cpu.set_flags(0, 0, hflag as u8, cflag as u8);
        let e: i16 = (cpu.fetched_data as u8).cast_signed() as i16;
        cpu.registers.set(cpu.curr_inst.reg_1,
            cpu.registers.read(cpu.curr_inst.reg_2).wrapping_add_signed(e));
    } else if cpu.dest_is_mem {
        if cpu.curr_inst.reg_2.is_16bit() {
            ctx.incr_cycle();
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
    let addr = cpu.registers.pc.wrapping_add_signed(rel as i16);
    goto_addr(cpu, bus, ctx, addr, false);
}

fn proc_call(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext) {
    goto_addr(cpu, bus, ctx, cpu.fetched_data, true);
}

fn proc_rst(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext) {
    goto_addr(cpu, bus, ctx, cpu.curr_inst.param as u16, true);
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
}

fn proc_reti(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext) {
    cpu.int_master_enabled = true;
    proc_ret(cpu, bus, ctx);
}

fn proc_di(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    cpu.int_master_enabled = false;
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
        cpu.registers.set(cpu.curr_inst.reg_1, data);
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

fn proc_inc(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext) {
    let mut val = cpu.fetched_data;
    
    if cpu.curr_inst.reg_1.is_16bit() {
        ctx.incr_cycle();
        val = val.wrapping_add(1);
    } else {
        val = (cpu.fetched_data as u8).wrapping_add(1) as u16;
    }

    if cpu.dest_is_mem {
        bus.write(cpu.registers.read(cpu.curr_inst.reg_1), val as u8);
    } else {
        cpu.registers.set(cpu.curr_inst.reg_1, val);
    }

    if !cpu.curr_inst.reg_1.is_16bit() {
        cpu.set_flags((val == 0) as u8, 0, if (val & 0x0F) == 0 {1} else {0}, BIT_IGNORE);
    }
}

fn proc_dec(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext) {
    let mut val = cpu.fetched_data;
    
    if cpu.curr_inst.reg_1.is_16bit() {
        ctx.incr_cycle();
    }

    if cpu.dest_is_mem {
        let val = (cpu.fetched_data as u8).wrapping_sub(1);
        bus.write(cpu.registers.read(cpu.curr_inst.reg_1), val);
    } else {
        val = val.wrapping_sub(1);
        cpu.registers.set(cpu.curr_inst.reg_1, val);
    }

    if !cpu.curr_inst.reg_1.is_16bit() {
        cpu.set_flags((val == 0) as u8, 1, ((val & 0x0F) == 0xF) as u8, BIT_IGNORE);
    }
}

fn proc_add(cpu: &mut CPU, _bus: &mut Interconnect, ctx: &mut EmuContext) {
    let mut value:u16; 

    if cpu.curr_inst.reg_1.is_16bit() {
        value = cpu.registers.read(cpu.curr_inst.reg_1).wrapping_add(cpu.fetched_data);
        ctx.incr_cycle();
    } else {
        value = (cpu.registers.read(cpu.curr_inst.reg_1) as u8).wrapping_add(cpu.fetched_data as u8) as u16;
    }

    if cpu.curr_inst.reg_1 == RegType::SP {
        let e = (cpu.fetched_data as u8).cast_signed();
        value = cpu.registers.read(cpu.curr_inst.reg_1).wrapping_add_signed(e as i16);
    }

    let mut z = (value == 0) as u8;
    let mut h = ((cpu.registers.read(cpu.curr_inst.reg_1) & 0xF) + (cpu.fetched_data & 0xF)) > 0xF;
    let mut c = ((cpu.registers.read(cpu.curr_inst.reg_1) & 0xFF) + (cpu.fetched_data & 0xFF)) > 0xFF;

    if cpu.curr_inst.reg_1.is_16bit() && cpu.curr_inst.reg_1 != RegType::SP {
        z = BIT_IGNORE;
        h  = ((cpu.registers.read(cpu.curr_inst.reg_1) & 0xFFF) + (cpu.fetched_data & 0xFFF)) > 0xFFF;
        (_, c) = cpu.registers.read(cpu.curr_inst.reg_1).overflowing_add(cpu.fetched_data)
    }

    if cpu.curr_inst.reg_1 == RegType::SP {
        z = 0;
    }

    cpu.registers.set(cpu.curr_inst.reg_1, value);
    cpu.set_flags(z, 0, h as u8, c as u8);
}

fn proc_adc(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    let a = cpu.registers.a;
    let c = cpu.c_flag() as u8;
    let (sum, overflow1) = a.overflowing_add(cpu.fetched_data as u8);
    let (sum, overflow2) = sum.overflowing_add(c);

    cpu.registers.a = sum;
    cpu.set_flags((sum == 0) as u8, 0, 
        ((a & 0xF) + c + (cpu.fetched_data as u8 & 0xF) > 0xF) as u8,
        (overflow1 || overflow2) as u8);
}

fn proc_sub(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    let (value, c) = cpu.registers.a.overflowing_sub(cpu.fetched_data as u8);
    let h = cpu.fetched_data as u8 & 0xF > (cpu.registers.a & 0xF);

    cpu.registers.a = value;
    cpu.set_flags((value == 0) as u8, 1, h as u8, c as u8);
}

fn proc_sbc(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    let a = cpu.registers.a;
    let u = cpu.fetched_data as u8;
    let c = cpu.c_flag() as u8;

    let (value, overflow1) = a.overflowing_sub(u);
    let (value, overflow2) = value.overflowing_sub(c);

    let h = ((u + c) & 0xF) > (a & 0xF);

    cpu.registers.a = value;
    cpu.set_flags((value == 0) as u8, 1, h as u8, (overflow1 || overflow2) as u8);
}

fn proc_and(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    cpu.registers.a &= cpu.fetched_data as u8;
    cpu.set_flags((cpu.registers.a == 0) as u8, 0, 1, 0);
}

fn proc_xor(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    cpu.registers.a ^= cpu.fetched_data as u8;
    cpu.set_flags((cpu.registers.a == 0) as u8, 0, 0, 0);
}

fn proc_or(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    cpu.registers.a |= (cpu.fetched_data & 0xFF) as u8;
    cpu.set_flags((cpu.registers.a == 0) as u8, 0, 0, 0);
}

fn proc_cp(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    let (n, overflow) = cpu.registers.a.overflowing_sub(cpu.fetched_data as u8);
    let h = (cpu.registers.a & 0xF) < (cpu.fetched_data as u8 & 0xF);

    cpu.set_flags((n == 0) as u8, 1, h as u8, overflow as u8);
}

fn proc_cb(cpu: &mut CPU, bus: &mut Interconnect, ctx: &mut EmuContext) {
    let op = cpu.fetched_data as u8;
    let register = decode_register(op & 0b111).unwrap();

    let bit = (op >> 3) & 0b111;
    let bit_op = (op >> 6) & 0b11;

    let mut reg_val = cpu.registers.read_reg8(bus, register);

    ctx.incr_cycle();

    if register == RegType::HL {
        ctx.incr_cycle();
        ctx.incr_cycle();
    }

    match bit_op {
        1 => { // BIT
            cpu.set_flags(((reg_val & (1 << bit)) == 0) as u8, 0, 1, BIT_IGNORE)
        }, 
        2 => { // RES
            reg_val &= !(1 << bit);
            cpu.registers.set_reg8(bus, register, reg_val)
        },
        3 => { // SET
            reg_val |= 1 << bit;
            cpu.registers.set_reg8(bus, register, reg_val)
        },
        0 => { // OTHER
            let cflag = cpu.c_flag() as u8;
            match bit {
                0 => { // RLC
                    let mut set_c = false;
                    let mut value = (reg_val << 1) & 0xFF;

                    if reg_val & (1 << 7) != 0 {
                        value |= 1;
                        set_c = true
                    }
                    cpu.registers.set_reg8(bus, register, value);
                    cpu.set_flags((value == 0) as u8, 0, 0, set_c as u8);
                },
                1 => { // RRC
                    let old = reg_val;
                    reg_val >>= 1;
                    reg_val |= old << 7;

                    cpu.registers.set_reg8(bus, register, reg_val);
                    cpu.set_flags((reg_val == 0) as u8, 0, 0, old & 1);
                }
                2 => { // RL
                    let old = reg_val;
                    reg_val <<= 1;
                    reg_val |= cflag;

                    cpu.registers.set_reg8(bus, register, reg_val);
                    cpu.set_flags((reg_val == 0) as u8, 0, 0, ((old & 0x80) != 0) as u8);
                },
                3 => { // RR
                    let old = reg_val;
                    reg_val >>= 1;
                    reg_val |= cflag << 7;

                    cpu.registers.set_reg8(bus, register, reg_val);
                    cpu.set_flags((reg_val == 0) as u8, 0, 0, old & 1);
                },
                4 => { // SLA,
                    let old = reg_val;
                    reg_val <<= 1;

                    cpu.registers.set_reg8(bus, register, reg_val);
                    cpu.set_flags((reg_val == 0) as u8, 0, 0, ((old & 0x80) != 0) as u8);
                }
                5 => { // SRA,
                    let u = (reg_val.cast_signed() >> 1) as u8;

                    cpu.registers.set_reg8(bus, register, u);
                    cpu.set_flags((u == 0) as u8, 0, 0, reg_val & 1);
                } 
                6 => { // SWAP
                    reg_val = ((reg_val & 0xF0) >> 4) | ((reg_val & 0x0F) << 4);

                    cpu.registers.set_reg8(bus, register, reg_val);
                    cpu.set_flags((reg_val == 0) as u8, 0, 0, 0);
                },
                7 => { // SRL
                    let old = reg_val;
                    reg_val >>= 1;

                    cpu.registers.set_reg8(bus, register, reg_val);
                    cpu.set_flags((reg_val == 0) as u8, 0, 0, old & 1);
                },
                _ => panic!("Unknown CB-prefixed command {op}")
            }
        }
        _ => panic!("Invalid bit operator in CB")
    }
}

fn proc_rlca(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    let mut u = cpu.registers.a;
    let c = (u >> 7) & 1; // MSB
    u = u << 1 | c;
    cpu.registers.a = u;

    cpu.set_flags(0, 0, 0, c);
}

fn proc_rrca(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    let b = cpu.registers.a & 1; // LSB
    cpu.registers.a >>= 1;
    cpu.registers.a |= b << 7;
    
    cpu.set_flags(0, 0, 0, b);
}

fn proc_rla(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    let u = cpu.registers.a;
    let cflag = cpu.c_flag() as u8;
    let c = (u >> 7) & 1; // MSB

    cpu.registers.a = u << 1 | cflag;
    cpu.set_flags(0, 0, 0, c);
}

fn proc_rra(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    let carry = cpu.c_flag() as u8;
    let c = cpu.registers.a & 1; // LSB

    cpu.registers.a >>= 1;
    cpu.registers.a |= carry << 7;

    cpu.set_flags(0, 0, 0, c);
}

fn proc_stop(_cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    panic!("STOP instruction received!")
}

fn proc_daa(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    let mut a = cpu.registers.a;
    let mut adjust: u8 = 0;
    let mut cflag: u8 = cpu.c_flag() as u8;

    let (n, h, c) = (cpu.n_flag(), cpu.h_flag(), cpu.c_flag());
    
    if !n {
        if h || (a & 0x0F) > 9 {
            adjust |= 0x06;
        }
        if c || a > 0x99 {
            adjust |= 0x60;
            cflag = 1;
        }
        a = a.wrapping_add(adjust);
    } else {
        if h {
            adjust |= 0x06;
        }
        if c {
            adjust |= 0x60;
        }
        a = a.wrapping_sub(adjust);
    }

    cpu.registers.a = a;
    cpu.set_flags((a == 0) as u8, BIT_IGNORE, 0, cflag);
}

fn proc_cpl(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    cpu.registers.a = !cpu.registers.a;
    cpu.set_flags(BIT_IGNORE, 1, 1, BIT_IGNORE);
}

fn proc_scf(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    cpu.set_flags(BIT_IGNORE, 0, 0, 1);
}

fn proc_ccf(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    let cflag = !cpu.c_flag() as u8;
    cpu.set_flags(BIT_IGNORE, 0, 0, cflag);
}

fn proc_halt(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    cpu.halted = true;
}

fn proc_ei(cpu: &mut CPU, _bus: &mut Interconnect, _ctx: &mut EmuContext) {
    cpu.enabling_ime = true;
}