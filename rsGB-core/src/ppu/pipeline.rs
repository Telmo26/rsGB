use crate::{interconnect::Interconnect, ppu::{XRES, fetcher::FetchState, utils::{lcd_read_ly, lcd_read_scroll_x, lcd_read_win_x, lcd_read_win_y, lcdc_obj_enable, lcdc_obj_height, lcdc_win_enable}}};

use super::PPU;

impl PPU {
    pub(super) fn process_fifo(&mut self, bus: &mut Interconnect, framebuffer: &mut [u32], render: bool) {
        // Dummy fetch simulation: the dummy fetch lasts for 6 dots,
        // so instead of warming up the pipeline I just skip the first
        // few dots. This works since the pixels are discarded anyways.
        // The first Mode 3 dot is 81 so I want to skip until 87 
        if self.line_ticks < 87 { 
            return ;
        }

        self.check_window_trigger(bus);

        // self.check_sprite_displayed(bus);

        if self.fetcher.is_fetching_sprite() {
            if self.bgw_fifo.len() == 0 
                && self.fetcher.bg_state != FetchState::Push 
            {
                // We add dots until the bg fifo is not empty basically
                self.fetcher.fetch_bgw(bus);
                if let Some(pixels) = self.fetcher.push_bgw(bus) {
                    for pixel in pixels {
                        self.bgw_fifo.push_back(pixel).unwrap();
                    }
                }

                return ;
            }

            self.fetcher.fetch(bus);
            if let Some(data) = self.fetcher.push_obj(bus) {
                while self.obj_fifo.len() < 8 {
                    self.obj_fifo.push_back((u32::MAX, 0, true)).unwrap();
                }
                for i in 0..8 {
                    let (new_pixel, new_index, new_bg_priority) = data[i];

                    // Only merge non-transparent pixels
                    if new_index != 0 {
                        let (_old_pixel, old_index, _old_bg_priority) = self.obj_fifo[i];

                        // X-Coordinate Priority:
                        // If the FIFO slot is empty (old_index 0), this new sprite wins.
                        // If the slot is *already* full, the old sprite (which had a
                        // lower X-coordinate) wins, and this new pixel is discarded.
                        if old_index == 0 {
                            self.obj_fifo[i] = (new_pixel, new_index, new_bg_priority);
                        }
                    }
                }
            }

            return ;
        }

        // println!("dot={} fetch={:?} fifo={} output={} window={}",
        //     self.line_ticks,
        //     self.fetcher.bg_state,
        //     self.bgw_fifo.len(),
        //     self.bgw_fifo.len() > 0,
        //     self.fetcher.is_window_mode()
        // );

        // Background/Window pixel fetcher
        self.fetcher.fetch(bus);

        // According to Pandocs, the FIFO tries to always hold at least 8 pixels
        if self.bgw_fifo.len() < 8 
            && let Some(pixels) = self.fetcher.push_bgw(bus) 
        {
            for pixel in pixels {
                self.bgw_fifo.push_back(pixel).unwrap();
            }
        }

        // According to Pandocs, 8 pixels are required for the Pixel
        // Rendering Operation to take place
        if self.bgw_fifo.len() > 0 {
            let (bgw_pixel, bgw_index) = self.bgw_fifo.pop_front().unwrap();

            if !self.fetcher.is_window_mode() {
                let scx = lcd_read_scroll_x(bus);
                
                // Discard the first (SCX % 8) pixels
                if self.current_x < (scx % 8) {
                    self.current_x += 1;
                    return;
                }
            }

            let (obj_pixel, obj_index, bg_priority) = self.obj_fifo.pop_front()
                .unwrap_or((u32::MAX, 0, true));

            let pixel = if obj_index == 0 || (bg_priority && bgw_index != 0) {
                bgw_pixel
            } else {
                obj_pixel
            };

            let x = self.screen_x as usize + lcd_read_ly(bus) as usize * XRES;
            
            if render {
                framebuffer[x] = pixel;
            }
            self.screen_x += 1;
        }
    }

    pub fn pipeline_reset(&mut self) {
        self.screen_x = 0;
        self.current_x = 0;

        self.bgw_fifo.clear();
        self.obj_fifo.clear();
        self.fetched_sprites.fill(false);
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
        if wx < 7 && self.screen_x == 0     // Window starts before or at screen edge
            || self.screen_x + 7 >= wx      // Normal case: window starts mid-screen
        {
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
        self.visible_sprites.clear();
    }

    pub fn frame_complete(&mut self) {
        // Reset window line counter at the start of each frame
        self.fetcher.reset_window_line();
    }

    pub fn oam_fetch(&mut self, bus: &mut Interconnect) {
        if self.visible_sprites.len() < 10 && self.line_ticks % 2 == 1 {
            // It takes two ticks to read a single OAM entry
            let ly = lcd_read_ly(bus);
            let sprite_height = lcdc_obj_height(bus);

            let index = ((self.line_ticks - 1) / 2) as u8;
            // println!("Current OAM index: {index}");
            let obj = bus.oam_sprite(index);

            // if obj.x == 0 {
            //     return;
            // }

            if obj.y <= ly + 16 && obj.y + sprite_height > ly + 16 {
                // This sprite is on the current line
                self.visible_sprites.push(obj);
            }
        }
    }

    fn check_sprite_displayed(&mut self, bus: &mut Interconnect) {
        if lcdc_obj_enable(bus) && !self.fetcher.is_fetching_sprite() {
            for (index, sprite) in self.visible_sprites.iter().enumerate() {
                let sprite_x = sprite.x.saturating_sub(8);

                if !self.fetched_sprites[index] 
                    && self.screen_x == sprite_x 
                {
                    self.fetcher.trigger_sprite_fetching(*sprite);
                    self.fetched_sprites[index] = true;
                    break;
                }
            }
        }
    }
}