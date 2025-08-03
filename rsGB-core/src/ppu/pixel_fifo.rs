use std::collections::VecDeque;

use crate::{interconnect::Interconnect, ppu::{utils::{lcd_read_ly, lcd_read_scroll_x, lcd_read_scroll_y, lcdc_bg_map_area, lcdc_bgw_data_area, lcdc_bgw_enable}, PIXELS, XRES}};

pub(in crate::ppu) enum FetchState {
    Tile,
    Data0,
    Data1,
    Idle,
    Push,
}

pub(in crate::ppu) struct PixelFifo {
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

    fn add(&mut self, bus: &mut Interconnect) -> bool {
        if self.pixel_fifo.len() > 8 {
            return false
        }
        let x = (self.fetch_x as u16).cast_signed() - (8 - lcd_read_scroll_x(bus) % 8) as i16;

        for i in 0..8 {
            let bit: u8 = 7 - i;
            let low = ((self.bgw_fetch_data[1] & (1 << bit)) != 0) as u8;

            let high = ((self.bgw_fetch_data[2] & (1 << bit)) != 0) as u8;

            let color = bus.lcd_bg_colors()[(high << 1  | low) as usize];

            if x >= 0 {
                self.push(color);
                self.fifo_x += 1;
            }
        }
        true
    }

    fn fetch(&mut self, bus: &mut Interconnect) {
        match self.current_state {
            FetchState::Tile => {
                if lcdc_bgw_enable(bus) {
                    let bg_map_area = lcdc_bg_map_area(bus);
                    self.bgw_fetch_data[0] = bus.read( bg_map_area + 
                        (self.map_x/8) as u16 + ((self.map_y / 8) as u16) * 32);
                    
                    if lcdc_bgw_data_area(bus) == 0x8800 {
                        self.bgw_fetch_data[0] = self.bgw_fetch_data[0].wrapping_add(128);
                    }
                }
                self.current_state = FetchState::Data0;
                self.fetch_x += 8;
            }
            FetchState::Data0 => {
                let bgw_data_area = lcdc_bgw_data_area(bus);
                self.bgw_fetch_data[1] = bus.read(bgw_data_area + 
                    self.bgw_fetch_data[0] as u16 * 16 + self.tile_y as u16);

                self.current_state = FetchState::Data1;
            }
            FetchState::Data1 => {
                let bgw_data_area = lcdc_bgw_data_area(bus);
                self.bgw_fetch_data[2] = bus.read(bgw_data_area + 
                    self.bgw_fetch_data[0] as u16 * 16 + self.tile_y as u16 + 1);

                self.current_state = FetchState::Idle;
            }
            FetchState::Idle => {
                self.current_state = FetchState::Push;
            },
            FetchState::Push => {
                if self.add(bus) {
                    self.current_state = FetchState::Tile;
                }
            },
        }
    }

    fn push_pixel(&mut self, bus: &mut Interconnect, video_buffer: &mut [u32; PIXELS]) {
        if self.pixel_fifo.len() > 8 {
            let pixel_data = self.pop();

            if self.line_x >= (lcd_read_scroll_x(bus) % 8) {
                let x = self.pushed_x as usize + lcd_read_ly(bus) as usize * XRES;
                video_buffer[x] = pixel_data;

                self.pushed_x += 1;
            }

            self.line_x += 1;
        }
    }

    pub fn process(&mut self, bus: &mut Interconnect, video_buffer: &mut [u32; PIXELS], line_ticks: u32) {
        self.map_y = lcd_read_ly(bus).wrapping_add(lcd_read_scroll_y(bus));
        self.map_x = self.fetch_x.wrapping_add(lcd_read_scroll_x(bus)); // Fetched X + SCROLL_X
        self.tile_y = (lcd_read_ly(bus).wrapping_add(lcd_read_scroll_y(bus)) % 8).wrapping_mul(2);

        if line_ticks % 2 == 0 {
            self.fetch(bus);
        }

        self.push_pixel(bus, video_buffer);
    }

    pub fn reset(&mut self) {
        self.pixel_fifo.clear();
    }
}