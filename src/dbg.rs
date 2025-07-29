use crate::interconnect::Interconnect;

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
        if bus.read(0xFF02) == 0x81 {
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
}