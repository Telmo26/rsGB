use crate::NO_IMPL;




// 0x0000 - 0x3FFF : ROM Bank 0
// 0x4000 - 0x7FFF : ROM Bank 1 - Switchable
// 0x8000 - 0x97FF : CHR RAM
// 0x9800 - 0x9BFF : BG Map 1
// 0x9C00 - 0x9FFF : BG Map 2
// 0xA000 - 0xBFFF : Cartridge RAM
// 0xC000 - 0xCFFF : RAM Bank 0
// 0xD000 - 0xDFFF : RAM Bank 1-7 - switchable - Color only
// 0xE000 - 0xFDFF : Reserved - Echo RAM
// 0xFE00 - 0xFE9F : Object Attribute Memory
// 0xFEA0 - 0xFEFF : Reserved - Unusable
// 0xFF00 - 0xFF7F : I/O Registers
// 0xFF80 - 0xFFFE : Zero Page

struct Bus {}

impl Bus {
    pub fn new() -> Bus {
        Bus {}
    }

    pub fn read(&self, address: u16) -> u8 {
        // ROM only for now
        match address {
            0x0000..=0x3FFF => cartridge::read(address),
            _ => NO_IMPL(),
        }
    }

    pub fn write(&self, address: u16, value: u8) {
        // ROM only for now
        match address {
            0x0000..=0x3FFF => cartridge::write(address),
            _ => NO_IMPL(),
        }
    }
}