use crate::cart::CartridgeInternals;

pub struct ROM {
    rom_data: Vec<u8>,
}

impl ROM {
    pub fn new(rom_data: Vec<u8>) -> ROM {
        ROM { rom_data }
    }
}

impl CartridgeInternals for ROM {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..0x8000 => self.rom_data[address as usize],
            _ => unreachable!()
        }
    }

    fn write(&mut self, _address: u16, _value: u8) {}

    fn need_save(&mut self) -> bool { false }

    fn save(&self, _save_path: &str) {}

    fn load_save(&mut self, _save_path: &str) {}
}