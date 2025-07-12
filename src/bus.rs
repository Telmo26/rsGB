struct Bus {}

impl Bus {
    pub fn new() -> Bus {
        Bus {}
    }

    pub fn read(&self, address: u16) -> u8 {
        0
    }

    pub fn write(&self, address: u16, value: u8) {

    }
}