use std::collections::VecDeque;

use crate::{interconnect::{Interconnect}, ppu::{utils::{lcd_read_ly, lcd_read_scroll_x, lcd_read_scroll_y, lcd_read_win_x, lcd_read_win_y, lcdc_bg_map_area, lcdc_bgw_data_area, lcdc_bgw_enable, lcdc_obj_enable, lcdc_obj_height, lcdc_win_enable, lcdc_win_map_area}, PPU, XRES, YRES}};

pub enum FetchState {
    Tile,
    Data0,
    Data1,
    Idle,
    Push,
}

pub struct PixelFifo {
    pub current_state: FetchState,
    pixel_fifo: VecDeque<u32>,
    pub line_x: u8,
    pub pushed_x: u8,
    pub fetch_x: u8,
    pub bgw_fetch_data: [u8; 3],
    pub fetch_entry_data: [u8; 6],
    pub map_y: u8,
    pub map_x: u8,
    pub tile_y: u8,
    pub fifo_x: u8,
}

impl PixelFifo {
    pub fn new() -> PixelFifo {
        PixelFifo {
            current_state: FetchState::Tile,
            pixel_fifo: VecDeque::with_capacity(9),
            line_x: 0,
            pushed_x: 0,
            fetch_x: 0,
            bgw_fetch_data: [0; 3],
            fetch_entry_data: [0; 6],
            map_y: 0,
            map_x: 0,
            tile_y: 0,
            fifo_x: 0,
        }
    }

    fn push(&mut self, value: u32) {
        self.pixel_fifo.push_back(value);
    }

    fn pop(&mut self) -> u32 {
        self.pixel_fifo.pop_front()
            .expect("Error in pixel FIFO")
    }

    fn reset(&mut self) {
        self.pixel_fifo.clear();
    }

    fn len(&self) -> usize {
        self.pixel_fifo.len()
    }
}

impl PPU {
    fn pipeline_add(&mut self, bus: &mut Interconnect) -> bool {
        if self.pixel_fifo.len() > 8 {
            return false
        }
        let x = (self.pixel_fifo.fetch_x as i16) - 8 + (lcd_read_scroll_x(bus) % 8) as i16;

        for i in 0..8 {
            let bit: u8 = 7 - i;
            let low = ((self.pixel_fifo.bgw_fetch_data[1] & (1 << bit)) != 0) as u8;

            let high = ((self.pixel_fifo.bgw_fetch_data[2] & (1 << bit)) != 0) as u8;

            let mut color = bus.lcd_bg_colors()[(high << 1  | low) as usize];

            if !lcdc_bgw_enable(bus) {
                color = bus.lcd_bg_colors()[0];
            }

            if lcdc_obj_enable(bus) {
                color = self.pipeline_fetch_sprite_pixels(bus, color, high << 1 | low)
            }

            if x >= 0 {
                self.pixel_fifo.push(color);
                self.pixel_fifo.fifo_x += 1;
            }
        }
        true
    }

    fn pipeline_fetch_sprite_pixels(&self, bus: &mut Interconnect, mut color: u32, bg_color: u8) -> u32 {        
        for i in 0..self.fetched_entries.len() {
            let sprite = self.fetched_entries[i];
            let sp_x = (sprite.x as i16) - 8 + ((lcd_read_scroll_x(bus) % 8) as i16);
            if (sp_x + 8) < self.pixel_fifo.fifo_x as i16 {
                // Past pixel point
                continue;
            }

            let offset = (self.pixel_fifo.fifo_x as i16) - sp_x;

            if offset < 0 || offset > 7 {
                // Out of bounds
                continue;
            }

            let mut bit = 7 - (offset as u8);
            if sprite.x_flip() {
                bit = offset as u8;
            }

            let low = (self.pixel_fifo.fetch_entry_data[i * 2] & (1 << bit) != 0) as u8;
            let high = (self.pixel_fifo.fetch_entry_data[i * 2 + 1] & (1 << bit) != 0) as u8;

            let bg_priority = sprite.bg_over_obj();
            
            let index = (high << 1 | low) as usize;

            if index == 0 {
                // Transparent
                continue;
            }

            if !bg_priority || bg_color == 0 {
                color = if sprite.palette_nb() { bus.lcd_sp2_colors()[index] } else { bus.lcd_sp1_colors()[index]};
                
                if index != 0 {
                    break
                }
            }
        }
        return color
    }

    fn pipeline_load_sprite_tile(&mut self, bus: &mut Interconnect) {
        for sprite_entry in &self.line_sprites {
            let sp_x = sprite_entry.x.wrapping_sub(8).wrapping_add(lcd_read_scroll_x(bus) % 8);

            if (sp_x >= self.pixel_fifo.fetch_x && sp_x < self.pixel_fifo.fetch_x + 8) ||
                (sp_x.wrapping_add(8) >= self.pixel_fifo.fetch_x && sp_x.wrapping_add(8) < self.pixel_fifo.fetch_x + 8) {
                self.fetched_entries.push(*sprite_entry);
            }

            if self.fetched_entries.len() > 2 {
                break;
            }
        }
    }

    fn pipeline_load_sprite_data(&mut self, bus: &mut Interconnect, offset: u8) {
        let cur_y = lcd_read_ly(bus);
        let sprite_height = lcdc_obj_height(bus);

        for i in 0..self.fetched_entries.len() {
            let mut tile_y = ((cur_y + 16) - self.fetched_entries[i].y) * 2;

            if self.fetched_entries[i].y_flip() {
                tile_y = (sprite_height * 2) - 2 - tile_y;
            }

            let mut tile_index = self.fetched_entries[i].tile as u16;

            if sprite_height == 16 {
                tile_index &= !0b1
            }

            self.pixel_fifo.fetch_entry_data[(i * 2) + offset as usize] = bus.read(0x8000 + (tile_index * 16) + tile_y as u16  + offset as u16);
        }
    }

    fn pipeline_load_window_tile(&mut self, bus: &mut Interconnect) {
        if !self.window_visible(bus) {
            return;
        }

        let window_y = lcd_read_win_y(bus);
        let window_x = lcd_read_win_x(bus);
        let ly = lcd_read_ly(bus);

        if self.pixel_fifo.fetch_x.wrapping_add(7) >= window_x &&
            self.pixel_fifo.fetch_x.wrapping_add(7) < window_x.wrapping_add(YRES as u8).wrapping_add(14) {

            if ly >= window_y && ly < window_y.wrapping_add(XRES as u8) {
                let w_tile_y = (self.window_line / 8) as u16;

                let map_area = lcdc_win_map_area(bus);

                let address = map_area +
                    ((self.pixel_fifo.fetch_x as u16 + 7 - window_x as u16) / 8) +
                    w_tile_y.wrapping_mul(32) ;

                self.pixel_fifo.bgw_fetch_data[0] = bus.read(address);
            }
        }
    }

    fn pipeline_fetch(&mut self, bus: &mut Interconnect) {
        match self.pixel_fifo.current_state {
            FetchState::Tile => {
                self.fetched_entries.clear();

                if lcdc_bgw_enable(bus) {
                    let bg_map_area = lcdc_bg_map_area(bus);
                    let address = bg_map_area + (self.pixel_fifo.map_x/8) as u16 + ((self.pixel_fifo.map_y / 8) as u16) * 32;

                    self.pixel_fifo.bgw_fetch_data[0] = bus.read(address);
                    
                    self.pipeline_load_window_tile(bus);

                    if lcdc_bgw_data_area(bus) == 0x8800 {
                        self.pixel_fifo.bgw_fetch_data[0] = self.pixel_fifo.bgw_fetch_data[0].wrapping_add(128);
                    }
                }

                if lcdc_obj_enable(bus) && !self.line_sprites.is_empty() {
                    self.pipeline_load_sprite_tile(bus);
                }
                self.pixel_fifo.current_state = FetchState::Data0;
                self.pixel_fifo.fetch_x += 8;
            }
            FetchState::Data0 => {
                let bgw_data_area = lcdc_bgw_data_area(bus);
                self.pixel_fifo.bgw_fetch_data[1] = bus.read(bgw_data_area + 
                    self.pixel_fifo.bgw_fetch_data[0] as u16 * 16 + self.pixel_fifo.tile_y as u16);
                
                self.pipeline_load_sprite_data(bus, 0);

                self.pixel_fifo.current_state = FetchState::Data1;
            }
            FetchState::Data1 => {
                let bgw_data_area = lcdc_bgw_data_area(bus);
                self.pixel_fifo.bgw_fetch_data[2] = bus.read(bgw_data_area + 
                    self.pixel_fifo.bgw_fetch_data[0] as u16 * 16 + self.pixel_fifo.tile_y as u16 + 1);

                self.pipeline_load_sprite_data(bus, 1);

                self.pixel_fifo.current_state = FetchState::Idle;
            }
            FetchState::Idle => {
                self.pixel_fifo.current_state = FetchState::Push;
            },
            FetchState::Push => {
                if self.pipeline_add(bus) {
                    self.pixel_fifo.current_state = FetchState::Tile;
                }
            },
        }
    }

    fn pipeline_push_pixel(&mut self, bus: &mut Interconnect) {
        if self.pixel_fifo.len() > 8 {
            let pixel_data = self.pixel_fifo.pop();

            if self.pixel_fifo.line_x >= (lcd_read_scroll_x(bus) % 8) {
                let x = self.pixel_fifo.pushed_x as usize + lcd_read_ly(bus) as usize * XRES;
                
                let mut framebuffer = self.framebuffer.lock().unwrap();
                framebuffer[x] = pixel_data;

                self.pixel_fifo.pushed_x += 1;
            }

            self.pixel_fifo.line_x += 1;
        }
    }

    pub fn pipeline_process(&mut self, bus: &mut Interconnect) {
        self.pixel_fifo.map_y = lcd_read_ly(bus).wrapping_add(lcd_read_scroll_y(bus));
        self.pixel_fifo.map_x = self.pixel_fifo.fetch_x.wrapping_add(lcd_read_scroll_x(bus)); // Fetched X + SCROLL_X
        self.pixel_fifo.tile_y = (self.pixel_fifo.map_y % 8).wrapping_mul(2);

        if self.line_ticks % 2 == 0 {
            self.pipeline_fetch(bus);
        }

        self.pipeline_push_pixel(bus);
    }

    pub fn pipeline_reset(&mut self) {
        self.pixel_fifo.reset();
    }

    pub fn window_visible(&self, bus: &mut Interconnect) -> bool {
        lcdc_win_enable(bus) && 
            lcd_read_win_x(bus) < XRES as u8 + 7 &&
            lcd_read_win_y(bus) < YRES as u8
    }
}