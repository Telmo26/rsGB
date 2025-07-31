pub struct PPU {
    vram: [u8; 0x2000],
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            vram: [0; 0x2000],
        }
    }

    pub fn tick(&self) {}
}
