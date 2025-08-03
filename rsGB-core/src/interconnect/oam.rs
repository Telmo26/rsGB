#[derive(Debug, Clone, Copy)]
pub struct OAMEntry {
    pub y: u8,
    pub x: u8,
    pub(crate) tile: u8,
    pub(crate) flags: u8,
}

/*
Flags: 
 Bit7   BG and Window over OBJ (0=No, 1=BG and Window colors 1-3 over the OBJ)
 Bit6   Y flip          (0=Normal, 1=Vertically mirrored)
 Bit5   X flip          (0=Normal, 1=Horizontally mirrored)
 Bit4   Palette number  **Non CGB Mode Only** (0=OBP0, 1=OBP1)
 Bit3   Tile VRAM-Bank  **CGB Mode Only**     (0=Bank 0, 1=Bank 1)
 Bit2-0 Palette number  **CGB Mode Only**     (OBP0-7)
*/

impl OAMEntry {
    pub fn new() -> OAMEntry {
        OAMEntry { y: 0, x: 0, tile: 0, flags: 0 }
    }

    pub fn write(&mut self, byte: u8, value: u8) {
        match byte {
            0 => self.y = value,
            1 => self.x = value,
            2 => self.tile = value,
            3 => self.flags = value,
            _ => panic!() // This can never happen
        }
    }

    pub fn read(&self, byte: u8) -> u8 {
        match byte {
            0 => self.y,
            1 => self.x,
            2 => self.tile,
            3 => self.flags,
            _ => panic!() // This can never happen
        }
    }

    pub fn bg_over_obj(&self) -> bool {
        self.flags & (1 << 7) != 0
    }

    pub fn y_flip(&self) -> bool {
        self.flags & (1 << 6) != 0
    }

    pub fn x_flip(&self) -> bool {
        self.flags & (1 << 5) != 0
    }

    pub fn palette_nb(&self) -> bool {
        self.flags & (1 << 4) != 0
    }

    pub fn _vram_bank(&self) -> bool {
        self.flags & (1 << 3) != 0
    }

    pub fn _cgb_palette_nb(&self) -> u8 {
        self.flags & 0b111
    }
}

