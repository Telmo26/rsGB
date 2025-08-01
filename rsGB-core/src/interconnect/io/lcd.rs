
use super::DMA;

const COLORS_DEFAULT : [u32; 4] = [0xFFFFFFFF, 0xFFAAAAAA, 0xFF555555, 0xFF000000]; 

enum LCDMode {
    HBlank,
    VBlank,
    OAM,
    XFer
}

enum StatSrc {
    HBlank = (1 << 3),
    VBlank = (1 << 4),
    OAM = (1 << 5),
    XFer = (1 << 6),
}

pub struct LCD {
    // Registers
    lcdc: u8,
    status: u8,
    scroll_y: u8,
    scroll_x: u8,
    ly: u8,
    ly_compare: u8,
    dma: u8,
    bg_palette: u8,
    obj_palette: [u8; 2],
    win_y: u8,
    win_x: u8,

    // Other data
    bg_colors: [u32; 4],
    sp1_colors: [u32; 4],
    sp2_colors: [u32; 4],
}

impl LCD {
    pub fn new() -> LCD {
        LCD {
            lcdc: 0x91,
            status: 0,
            scroll_y: 0,
            scroll_x: 0,
            ly: 0,
            ly_compare: 0,
            dma: 0,
            bg_palette: 0xFC,
            obj_palette: [0xFF; 2],
            win_y: 0,
            win_x: 0,

            bg_colors: COLORS_DEFAULT,
            sp1_colors: COLORS_DEFAULT,
            sp2_colors: COLORS_DEFAULT,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF40 => self.lcdc,
            0xFF41 => self.status,
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.ly,
            0xFF45 => self.ly_compare,
            0xFF46 => self.dma,
            0xFF47 => self.bg_palette,
            0xFF48 => self.obj_palette[0],
            0xFF49 => self.obj_palette[1],
            0xFF4A => self.win_x,
            0xFF4B => self.win_y,
            _ => panic!(),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF40 => self.lcdc = value,
            0xFF41 => self.status = value,
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF44 => self.ly = value,
            0xFF45 => self.ly_compare = value,
            0xFF46 => self.dma = value,
            0xFF47 => self.bg_palette = value,
            0xFF48 => self.obj_palette[0] = value,
            0xFF49 => self.obj_palette[1] = value,
            0xFF4A => self.win_x = value,
            0xFF4B => self.win_y = value,
            _ => panic!(),
        }

        if address == 0xFF47 {
            self.update_palette(value, 0)
        } else if address == 0xFF48 {
            self.update_palette(value & 0b11111100, 1)
        } else if address == 0xFF49 {
            self.update_palette(value & 0b11111100, 2)
        }
    }

    fn update_palette(&mut self, palette_data: u8, palette: u8) {
        let mut p_colors = &mut self.bg_colors;
        if palette == 0 {
            p_colors = &mut self.sp1_colors;
        } else {
            p_colors = &mut self.sp2_colors;
        }

        p_colors[0] = COLORS_DEFAULT[palette_data as usize & 0b11];
        p_colors[1] = COLORS_DEFAULT[(palette_data >> 2) as usize & 0b11];
        p_colors[2] = COLORS_DEFAULT[(palette_data >> 4) as usize & 0b11];
        p_colors[3] = COLORS_DEFAULT[(palette_data >> 6) as usize & 0b11];
    }

    fn lcdc_bgw_enable(&self) -> bool { (self.lcdc & 1) != 0 }

    fn lcdc_obj_enable(&self) -> bool { (self.lcdc & (1 << 1)) != 0 }

    fn lcdc_obj_height(&self) -> u8 {
        if self.lcdc & (1 << 2) != 0 {
            16
        } else {
            8
        }
    }

    fn lcdc_bg_map_area(&self) -> u16 {
        if self.lcdc & (1 << 3) != 0 {
            0x9C00
        } else {
            0x9800
        }
    }

    fn lcdc_bgw_data_area(&self) -> u16 {
        if self.lcdc & (1 << 4) != 0 {
            0x8000
        } else {
            0x8800
        }
    }

    fn lcdc_win_enable(&self) -> bool { (self.lcdc & (1 << 5)) != 0 }

    fn lcdc_win_map_area(&self) -> u16 {
        if self.lcdc & (1 << 6) != 0 {
            0x9800
        } else {
            0x9C00
        }
    }

    fn lcdc_lcd_enable(&self) -> bool { (self.lcdc & (1 << 7)) != 0 }

    fn status_mode(&self) -> LCDMode {
        match self.status & 0b11 {
            0 => LCDMode::HBlank,
            1 => LCDMode::VBlank,
            2 => LCDMode::OAM,
            _ => LCDMode::XFer,
        }
    }

    fn status_mode_set(&mut self, mode: LCDMode) {
        self.status &= !0b11;
        match mode {
            LCDMode::HBlank => self.status |= 0b00,
            LCDMode::VBlank => self.status |= 0b01,
            LCDMode::OAM => self.status |= 0b10,
            LCDMode::XFer => self.status |= 0b11, 
        }
    }

    fn status_lyc(&self) -> bool { self.status & (1 << 2) != 0 }

    fn status_lyc_set(&mut self, value: u8) { self.status |= value << 2 } 

    fn status_stat_int(&self, src: StatSrc) -> bool { self.status & (src as u8) != 0 }
}