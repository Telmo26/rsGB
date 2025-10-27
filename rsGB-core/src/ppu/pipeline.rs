use crate::{interconnect::Interconnect, ppu::{utils::{lcd_read_ly, lcd_read_scroll_x}, XRES}};

use super::PPU;

impl PPU {
    pub(super) fn process_fifo(&mut self, bus: &mut Interconnect, framebuffer: &mut [u32]) {
        if self.bgw_fifo.is_empty() {
            self.fetcher.fetch(bus);
            if let Some(pixels) = self.fetcher.push(bus) {
                self.bgw_fifo.extend(pixels);
            }
        }

        if !self.bgw_fifo.is_empty() {
            let pixel_data = self.bgw_fifo.pop_front().unwrap();

            let scx = lcd_read_scroll_x(bus);

            if self.current_x >= (scx % 8) {
                let x = self.pushed_x as usize + lcd_read_ly(bus) as usize * XRES;
                
                framebuffer[x] = pixel_data;
                self.pushed_x += 1;
            }

            self.current_x += 1;
        }
    }

    pub fn fetch_oam(&mut self, bus: &mut Interconnect) {
        self.fetcher.oam(bus)
    }

    pub fn pipeline_reset(&mut self) {
        self.pushed_x = 0;
        self.current_x = 0;

        self.bgw_fifo.clear();
        self.fetcher.reset();
    }
}