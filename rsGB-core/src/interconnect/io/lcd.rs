use crate::ColorMode;


const COLORS_DEFAULT_ARGB : [u32; 4] = [0xFFFFFFFF, 0xFFAAAAAA, 0xFF555555, 0xFF000000];
const COLORS_DEFAULT_RGBA : [u32; 4] = [0xFFFFFFFF, 0xAAAAAAFF, 0x555555FF, 0x000000FF];

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
    color_mode: ColorMode,
    pub(crate) bg_colors: [u32; 4],
    pub(crate) sp1_colors: [u32; 4],
    pub(crate) sp2_colors: [u32; 4],
}

impl LCD {
    pub fn new(color_mode: ColorMode) -> LCD {
        let default = match color_mode {
            ColorMode::ARGB => COLORS_DEFAULT_ARGB,
            ColorMode::RGBA => COLORS_DEFAULT_RGBA,
        };
        LCD {
            lcdc: 0x91,
            status: 0x02,
            scroll_y: 0,
            scroll_x: 0,
            ly: 0,
            ly_compare: 0,
            dma: 0,
            bg_palette: 0xFC,
            obj_palette: [0xFF; 2],
            win_y: 0,
            win_x: 0,

            color_mode,
            bg_colors: default,
            sp1_colors: default,
            sp2_colors: default,
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
            0xFF4A => self.win_y,
            0xFF4B => self.win_x,
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
            0xFF4A => self.win_y = value,
            0xFF4B => self.win_x = value,
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
        if palette == 1 {
            p_colors = &mut self.sp1_colors;
        } else if palette == 2 {
            p_colors = &mut self.sp2_colors;
        }

        let colors = match self.color_mode {
            ColorMode::ARGB => &COLORS_DEFAULT_ARGB,
            ColorMode::RGBA => &COLORS_DEFAULT_RGBA,
        };

        p_colors[0] = colors[palette_data as usize & 0b11];
        p_colors[1] = colors[(palette_data >> 2) as usize & 0b11];
        p_colors[2] = colors[(palette_data >> 4) as usize & 0b11];
        p_colors[3] = colors[(palette_data >> 6) as usize & 0b11];
    }
}