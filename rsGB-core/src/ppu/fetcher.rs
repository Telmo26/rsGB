use crate::{interconnect::{Interconnect, OAMEntry}, ppu::utils::{lcd_read_ly, lcd_read_scroll_x, lcd_read_scroll_y, lcdc_bg_map_area, lcdc_bgw_data_area, lcdc_bgw_enable, lcdc_obj_height, lcdc_win_map_area}};

enum Step {
    First,
    Second,
}

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

pub(super) struct Fetcher {
    state: FetchState,
    mode: FetchMode,

    lx: u8,
    tile_address: u16,
    bgw_fetched_data: [u8; 3],
    data_address: u16,

    window_line: u8,

    obj_slots: Vec<OAMEntry>,
    oam_index: u8,
    current_obj: Option<OAMEntry>,
    oam_step: Step,

    pushed_x: u8, // The position of the last pixel that was pushed to the FIFO

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

            obj_slots: Vec::with_capacity(10),
            oam_index: 0,
            current_obj: None,
            oam_step: Step::First,

            pushed_x: 0,
        }
    }

    pub fn reset(&mut self) {
        self.state = FetchState::TileID(Step::First);
        self.mode = FetchMode::Background;

        self.lx = 0;
        self.pushed_x = 0;

        self.obj_slots.clear();
        self.oam_index = 0;
        self.current_obj = None;
        self.oam_step = Step::First;
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

    pub fn fetch(&mut self, bus: &mut Interconnect) {
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

    pub fn push(&mut self, bus: &mut Interconnect) -> Option<Vec<u32>> {
        if let FetchState::Push = self.state {
            let mut pixels = Vec::new();
            for i in 0..8 {
                let bit: u8 = 7 - i;
                let low = ((self.bgw_fetched_data[1] & (1 << bit)) != 0) as u8;

                let high = ((self.bgw_fetched_data[2] & (1 << bit)) != 0) as u8;

                let color = if lcdc_bgw_enable(bus) { 
                    bus.lcd_bg_colors()[(high << 1  | low) as usize]
                } else {                
                    bus.lcd_bg_colors()[0]
                };

                pixels.push(color);
            }
            self.pushed_x += 8;
            self.state = FetchState::TileID(Step::First);
            Some(pixels)
        } else {
            None
        }
    }

    pub fn oam(&mut self, bus: &mut Interconnect) {
        if self.obj_slots.len() < 10 {
            match self.oam_step {
                Step::First => {
                    self.current_obj = Some(bus.oam_sprite(self.oam_index));
                    self.oam_index += 1;
                    self.oam_step = Step::Second;
                },
                Step::Second => {
                    let cur_y = lcd_read_ly(bus);
                    let sprite_height = lcdc_obj_height(bus);

                    if let Some(obj) = self.current_obj {
                        if obj.x > 0 && obj.y <= cur_y + 16 && obj.y + sprite_height > cur_y + 16 { 
                            // x = 0 means invisible
                            // This sprite is on the current line
                            self.obj_slots.push(obj);
                        }
                    }
                }
            }
            
        }
    }
}