use crate::Devices;

pub struct PPU {

}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            
        }
    }

    pub fn tick(&self) {}

    pub fn oam_read(&self, dev: &mut Devices, address: u16) -> u8 {
        dev.bus_read(address)
    }

    pub fn oam_write(&self, dev: &mut Devices, address: u16, value: u8) {
        dev.bus_write(address, value);
    }

    pub fn vram_read(&self, dev: &mut Devices, address: u16) -> u8 {
        dev.bus_read(address)
    }

    pub fn vram_write(&self, dev: &mut Devices, address: u16, value: u8) {
        dev.bus_write(address, value);
    }
}
