use std::u32;

use crate::{interconnect::{Interconnect, OAMEntry}, ppu::utils::{lcd_read_ly, lcd_read_scroll_x, lcd_read_scroll_y, lcdc_bg_map_area, lcdc_bgw_data_area, lcdc_bgw_enable, lcdc_obj_height, lcdc_win_map_area}};

#[derive(Debug)]
enum Step {
    First,
    Second,
}

#[derive(Debug)]
enum FetchState {
    TileID(Step),
    TileRowLow(Step),
    TileRowHigh(Step),
    Push,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FetchMode {
    Background,
    Window,
}

#[derive(Debug)]
pub(super) struct Fetcher {
    state: FetchState,
    mode: FetchMode,

    pub lx: u8,
    tile_address: u16,
    bgw_fetched_data: [u8; 3],
    data_address: u16,

    window_line: u8,

    fetching_sprite: bool,
    current_sprite: Option<OAMEntry>,
    sprite_data: [u8; 2],

    pub pushed_x: u8, // The position of the last pixel that was pushed to the FIFO
}

impl Fetcher {
    pub fn new() -> Fetcher {
        Fetcher { 
            state : FetchState::TileID(Step::First), 
            mode: FetchMode::Background,

            lx: 0,
            tile_address: 0,
            bgw_fetched_data: [0; 3],
            data_address: 0, 

            window_line: 0,

            fetching_sprite: false,
            current_sprite: None,
            sprite_data: [0; 2],

            pushed_x: 0,
        }
    }

    pub fn reset(&mut self) {
        self.state = FetchState::TileID(Step::First);
        self.mode = FetchMode::Background;
        self.fetching_sprite = false;

        self.lx = 0;
        self.pushed_x = 0;
    }

    pub fn reset_window_line(&mut self) {
        self.window_line = 0;
    }

    pub fn switch_to_window(&mut self) {
        // When switching to window mid-scanline, reset fetch position
        self.mode = FetchMode::Window;
        self.lx = 0;
        self.state = FetchState::TileID(Step::First);
    }

    pub fn increment_window_line(&mut self) {
        if self.mode == FetchMode::Window {
            self.window_line += 1;
        }
    }

    pub fn is_window_mode(&self) -> bool {
        self.mode == FetchMode::Window
    }

    pub fn is_fetching_sprite(&self) -> bool {
        self.fetching_sprite
    }

    pub fn trigger_sprite_fetching(&mut self, sprite: OAMEntry) {
        self.fetching_sprite = true;
        self.current_sprite = Some(sprite);
        self.state = FetchState::TileRowLow(Step::First);
    }

    pub fn fetch(&mut self, bus: &mut Interconnect) {
        if self.fetching_sprite {
            self.fetch_sprite(bus);
        } else {
            self.fetch_bgw(bus);
        }
    }

    fn fetch_bgw(&mut self, bus: &mut Interconnect) {
        match self.state {
            FetchState::TileID(Step::First) => {                
                self.tile_address = if self.mode == FetchMode::Background {
                    let (ly, scx, scy) = (lcd_read_ly(bus), lcd_read_scroll_x(bus), lcd_read_scroll_y(bus));
                    let bg_map_area = lcdc_bg_map_area(bus);

                    bg_map_area +
                    ((ly.wrapping_add(scy) as u16 / 8) << 5) +
                    ((self.lx.wrapping_add(scx) as u16) / 8)
                } else {
                    let win_map_area = lcdc_win_map_area(bus);
                    win_map_area + 
                    (((self.window_line / 8) as u16) << 5) +
                    (self.lx as u16 / 8)
                };
                
                self.state = FetchState::TileID(Step::Second);
                self.lx += 8;
            }

            FetchState::TileID(Step::Second) => {
                self.bgw_fetched_data[0] = bus.read(self.tile_address);
                self.state = FetchState::TileRowLow(Step::First);
            }

            FetchState::TileRowLow(Step::First) => {
                let bgw_data_area = lcdc_bgw_data_area(bus);
                let tile_id = self.bgw_fetched_data[0];
                
                let tile_row = if self.mode == FetchMode::Background {
                    let (ly, scy) = (lcd_read_ly(bus), lcd_read_scroll_y(bus));
                    ly.wrapping_add(scy) % 8
                } else {
                    self.window_line % 8
                };

                self.data_address = if bgw_data_area == 0x8000 {
                    // Unsigned: 0x8000-0x8FFF, tile_id as u8
                    0x8000 + ((tile_id as u16) << 4) + ((tile_row as u16) << 1)
                } else {
                    // Signed: 0x9000 base, tile_id as i8
                    0x9000_u16.wrapping_add_signed((tile_id as i8 as i16) << 4) + ((tile_row as u16) << 1)
                };
                self.state = FetchState::TileRowLow(Step::Second);
            }

            FetchState::TileRowLow(Step::Second) => {
                self.bgw_fetched_data[1] = bus.read(self.data_address);
                self.state = FetchState::TileRowHigh(Step::First);
            }

            FetchState::TileRowHigh(Step::First) => { 
                self.data_address += 1;
                self.state = FetchState::TileRowHigh(Step::Second);
            }

            FetchState::TileRowHigh(Step::Second) => {
                self.bgw_fetched_data[2] = bus.read(self.data_address);
                self.state = FetchState::Push;
            }

            FetchState::Push => { }
        }
    }

    fn fetch_sprite(&mut self, bus: &mut Interconnect) {
        match self.state {
            FetchState::TileRowLow(Step::First) => {
                let ly = lcd_read_ly(bus);
                let sprite_height = lcdc_obj_height(bus);         
                let sprite = self.current_sprite.unwrap();

                let mut tile_y = ly + 16 - sprite.y;
                if sprite.y_flip() {
                    tile_y = sprite_height - 1 - tile_y;
                }

                let tile_index = if sprite_height == 16 {
                    sprite.tile as u16 & !0b1
                } else {
                    sprite.tile as u16
                };

                self.data_address = 0x8000 + (tile_index << 4) + ((tile_y as u16) << 1);
                self.state = FetchState::TileRowLow(Step::Second);
            }

            FetchState::TileRowLow(Step::Second) => {
                self.sprite_data[0] = bus.read(self.data_address);
                self.state = FetchState::TileRowHigh(Step::First);
            }

            FetchState::TileRowHigh(Step::First) => { 
                self.data_address += 1;
                self.state = FetchState::TileRowHigh(Step::Second);
            }

            FetchState::TileRowHigh(Step::Second) => {
                self.sprite_data[1] = bus.read(self.data_address);
                self.state = FetchState::Push;
            }

            _ => { 
                // println!("Fetcher: {:#?}", self);
            }
        }
    }

    // This function returns the color value for the background
    // and the index, to handle transparency
    pub fn push_bgw(&mut self, bus: &mut Interconnect) -> Option<Vec<(u32, u8)>> {
        if let FetchState::Push = self.state {
            let mut pixels = Vec::new();
            for i in 0..8 {
                let bit: u8 = 7 - i;
                let low = ((self.bgw_fetched_data[1] & (1 << bit)) != 0) as u8;

                let high = ((self.bgw_fetched_data[2] & (1 << bit)) != 0) as u8;

                let index = if lcdc_bgw_enable(bus) {
                    high << 1 | low
                } else { 0 };

                let color = bus.lcd_bg_colors()[index as usize];

                pixels.push((color, index));
            }
            self.pushed_x += 8;
            self.state = FetchState::TileID(Step::First);
            Some(pixels)
        } else {
            None
        }
    }

    // This function returns the pixel value, with the palette number
    // and the OBJ-to-BG priority flag
    pub fn push_obj(&mut self, bus: &mut Interconnect) -> Option<Vec<(u32, u8, bool)>> {
        if let FetchState::Push = self.state {
            let mut pixels = Vec::with_capacity(8);
            let sprite = self.current_sprite.unwrap();
            
            for i in 0..8 {
                let bit = if sprite.x_flip() { i } else { 7 - i };
                let low = (self.sprite_data[0] & (1 << bit) != 0) as u8;
                let high = (self.sprite_data[1] & (1 << bit) != 0) as u8;

                let bg_priority = sprite.bg_over_obj();
                let index = (high << 1 | low) as usize;
                let color = if index == 0 {
                    u32::MAX // This will never be read, since index = 0 means transparent
                } else if sprite.palette_nb() {
                    bus.lcd_sp2_colors()[index]
                } else {
                    bus.lcd_sp1_colors()[index]
                };
                pixels.push((color, index as u8, bg_priority));
            };
            self.fetching_sprite = false;
            self.current_sprite = None;
            self.state = FetchState::TileID(Step::First);
            return Some(pixels)
        }
        None        
    }
}