use crate::{cpu::{AddrMode, RegType, CPU}, interconnect::Interconnect};

pub struct Debugger {
    msg: [u8; 1024],
    msg_size: usize,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {
            msg: [0; 1024],
            msg_size: 0,
        }
    }

    pub fn update(&mut self, bus: &mut Interconnect) {
        if bus.read(0xFF02) & 0x83 == 0x81 {
            let c = bus.read(0xFF01);

            self.msg[self.msg_size] = c;
            self.msg_size += 1;

            bus.write(0xFF02, 0);
        }
    }

    pub fn print(&self) {
        if self.msg[0] != 0 {
            println!("DBG: {}", str::from_utf8(&self.msg).unwrap());
        }
    }

    pub fn debug_info(&mut self, cpu: &mut CPU, bus: &mut Interconnect, ticks: u64, previous_pc: u16) {
        let hl  = if cpu.curr_inst.mode == AddrMode::HLI_R || cpu.curr_inst.mode == AddrMode::R_HLI { 
            cpu.registers.read(RegType::HL) - 1
        } else if cpu.curr_inst.mode == AddrMode::HLD_R || cpu.curr_inst.mode == AddrMode::R_HLD {
            cpu.registers.read(RegType::HL) + 1
        } else {
            cpu.registers.read(RegType::HL)
        };
        
        let flags = format!(
            "Flags : {}{}{}{}",
            if cpu.registers.f & 1 << 7 != 0 { 'Z' } else { '-' },
            if cpu.registers.f & 1 << 6 != 0 { 'N' } else { '-' },
            if cpu.registers.f & 1 << 5 != 0 { 'H' } else { '-' },
            if cpu.registers.f & 1 << 4 != 0 { 'C' } else { '-' },
        );

        let inst_part = format!(
            "Ticks: {:08X} PC: {:04X} \t {} ({:02X} {:02X} {:02X})",
            ticks,
            previous_pc, 
            cpu.curr_inst.to_str(cpu), 
            cpu.curr_opcode, 
            bus.read(previous_pc + 1), 
            bus.read(previous_pc + 2),
        );

    
        let reg_part = format!(
            "A: {:02X} BC: {:02X}{:02X} DE: {:02X}{:02X} HL: {:04X} SP: {:04X}", 
            cpu.registers.a, 
            cpu.registers.b, cpu.registers.c,
            cpu.registers.d, cpu.registers.e, 
            hl,
            cpu.registers.sp
        );

        println!("{:<35} {}", inst_part, reg_part);
        println!("{:<32} {}", "", flags);

        self.update(bus);
        self.print();
    }
}