
pub struct IO {
    serial: [u8; 2]
}

impl IO {
    pub fn new() -> IO {
        IO { 
            serial: [0; 2],
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF01 => self.serial[0],
            0xFF02 => self.serial[1],
            _ => {
                println!("Read at address {address:X} not implemented!");
                0
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF01 => self.serial[0] = value,
            0xFF02 => self.serial[1] = value,
            _ => println!("Write at address {address:X} not implemented!"),
        }
    }
}