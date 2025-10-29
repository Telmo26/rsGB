use crate::{interconnect::Interconnect, ppu::{XRES, utils::{lcd_read_ly, lcd_read_scroll_x, lcd_read_win_x, lcd_read_win_y, lcdc_obj_height, lcdc_win_enable}}};

use super::PPU;

impl PPU {
    pub(super) fn process_fifo(&mut self, bus: &mut Interconnect, framebuffer: &mut [u32]) {
        self.check_window_trigger(bus);

        self.check_sprite_displayed();

        if self.bgw_fifo.is_empty() && !self.fetcher.is_fetching_sprite() {
            self.fetcher.fetch(bus);
            if let Some(pixels) = self.fetcher.push_bgw(bus) {
                self.bgw_fifo.extend(pixels);
            }
        }

        if self.fetcher.is_fetching_sprite() {
            println!("Fetching sprite data!");
            self.fetcher.fetch(bus);
            if let Some(data) = self.fetcher.push_obj(bus) {
                self.obj_fifo.extend(data);
                self.fetcher.reset_to_background();
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
       
        // Check if we've reached the window Y position
        if ly < wy {
            return;
        }

        // Check if we've reached the window X position
        // WX=0-6 are off-screen, WX=7 is at screen position 0
        if wx < 7 {
            // Window starts before or at screen edge
            if self.pushed_x == 0 {
                self.switch_to_window_mode();
            }
        } else if self.pushed_x + 7 >= wx {
            // Normal case: window starts mid-screen
            self.switch_to_window_mode();
        }
    }

    fn switch_to_window_mode(&mut self) {
        // Clear the FIFO when switching to window
        self.bgw_fifo.clear();
        
        // Switch fetcher to window mode
        self.fetcher.switch_to_window();
    }

    pub fn scanline_complete(&mut self) {
        // Increment window line counter if window was rendered this scanline
        self.fetcher.increment_window_line();
    }

    pub fn frame_complete(&mut self) {
        // Reset window line counter at the start of each frame
        self.fetcher.reset_window_line();
    }

    pub fn oam_fetch(&mut self, bus: &mut Interconnect) {
        if self.visible_sprites.len() < 10 && self.line_ticks % 2 == 0 {
            // It takes two ticks to read a single OAM entry
            let ly = lcd_read_ly(bus);
            let sprite_height = lcdc_obj_height(bus);

            let index = (self.line_ticks / 2) as u8;
            let obj = bus.oam_sprite(index);

            if obj.x == 0 {
                return;
            }

            if obj.y <= ly + 16 && obj.y + sprite_height > ly + 16 {
                // This sprite is on the current line
                self.visible_sprites.push(obj);
            }

            if self.visible_sprites.len() == 10 {
                self.visible_sprites.sort_by_key(|e| e.x);
            }
        }
    }

    fn check_sprite_displayed(&mut self) {
        for (index, sprite) in self.visible_sprites.iter().enumerate() {
            let sprite_x = sprite.x - 8;

            if !self.fetched_sprites[index] && sprite_x == self.pushed_x {
                self.fetcher.trigger_sprite_fetching(*sprite);
                self.fetched_sprites[index] = true;
                break;
            }
        }
    }
}