use crate::{interconnect::Interconnect, ppu::{XRES, utils::{lcd_read_ly, lcd_read_scroll_x, lcd_read_win_x, lcd_read_win_y, lcdc_win_enable}}};

use super::PPU;

impl PPU {
    pub(super) fn process_fifo(&mut self, bus: &mut Interconnect, framebuffer: &mut [u32]) {
        // self.check_window_trigger(bus);

        if self.bgw_fifo.is_empty() {
            self.fetcher.fetch(bus);
            if let Some(pixels) = self.fetcher.push(bus) {
                self.bgw_fifo.extend(pixels);
            }
        }

        if !self.bgw_fifo.is_empty() {
            let pixel_data = self.bgw_fifo.pop_front().unwrap();

            if !self.fetcher.is_window_mode() {
                let scx = lcd_read_scroll_x(bus);
                
                // Discard the first (SCX % 8) pixels
                if self.current_x < (scx % 8) {
                    self.current_x += 1;
                    return;
                }
            }

            let x = self.pushed_x as usize + lcd_read_ly(bus) as usize * XRES;
            
            framebuffer[x] = pixel_data;
            self.pushed_x += 1;
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

    fn check_window_trigger(&mut self, bus: &mut Interconnect) {
        if self.fetcher.is_window_mode() || !lcdc_win_enable(bus) {
            return;
        }

        let ly = lcd_read_ly(bus);
        let wy = lcd_read_win_y(bus);
        let wx = lcd_read_win_x(bus);

        // Window X is offset by 7 in hardware
        // Window becomes visible when pushed_x + 7 >= wx
        // This means when pushed_x >= wx - 7
        
        // Check if we've reached the window Y position
        if ly < wy {
            return;
        }

        // Check if we've reached the window X position
        // WX=0-6 are off-screen, WX=7 is at screen position 0
        if wx < 7 {
            // Window starts before or at screen edge
            if self.pushed_x == 0 {
                self.switch_to_window_mode(bus);
            }
        } else if self.pushed_x + 7 >= wx {
            // Normal case: window starts mid-screen
            self.switch_to_window_mode(bus);
        }
    }

    fn switch_to_window_mode(&mut self, bus: &mut Interconnect) {
        // Clear the FIFO when switching to window
        self.bgw_fifo.clear();
        
        // Switch fetcher to window mode
        self.fetcher.switch_to_window();
        
        // The current_x counter continues, but we don't apply SCX discard anymore
    }

    pub fn scanline_complete(&mut self) {
        // Increment window line counter if window was rendered this scanline
        self.fetcher.increment_window_line();
    }

    pub fn frame_complete(&mut self) {
        // Reset window line counter at the start of each frame
        self.fetcher.reset_window_line();
    }
}