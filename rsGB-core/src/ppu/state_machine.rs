use crate::{
    interconnect::{Interconnect, InterruptType}, ppu::utils::{lcd_read_ly, lcd_read_win_y, lcd_write_ly, lcdc_obj_height}, 
};

use super::{
    LINES_PER_FRAME, PPU, TICKS_PER_LINE, XRES, YRES,
    pipeline::FetchState,
    utils::{LCDMode, StatusSrc, status_stat_int, status_lyc_set, change_lcd_mode, status_mode_set}
};

impl PPU {
    fn load_line_sprites(&mut self, bus: &mut Interconnect) {
        let cur_y = lcd_read_ly(bus);
        let sprite_height = lcdc_obj_height(bus);

        for i in 0..40 {
            let e = bus.oam_sprite(i);

            if e.x == 0 { continue } // X = 0 means invisible

            if self.line_sprites.len() > 9 { 
                break 
            }

            if e.y <= cur_y + 16 && e.y + sprite_height > cur_y + 16 {
                // This sprite is on the current line
                self.line_sprites.push(e);
            }
        }
        self.line_sprites.sort_by_key(|e| e.x);
    }   
    pub fn hblank(&mut self, bus: &mut Interconnect) {
        if self.line_ticks >= TICKS_PER_LINE {
            let ly = self.increment_ly(bus);

            if ly >= YRES as u8 {
                change_lcd_mode(bus, LCDMode::VBlank);

                bus.request_interrupt(InterruptType::VBlank);

                if status_stat_int(bus, StatusSrc::VBlank) {
                    bus.request_interrupt(InterruptType::LcdStat);
                }
                self.current_frame += 1;
                self.new_frame = true;
            } else {
                status_mode_set(bus, LCDMode::OAM);
            }
            self.line_ticks = 0;
        }
    }

    pub fn vblank(&mut self, bus: &mut Interconnect) {
        if self.line_ticks >= TICKS_PER_LINE {
            let ly = self.increment_ly(bus);

            if ly >= LINES_PER_FRAME {
                change_lcd_mode(bus, LCDMode::OAM);
                bus.write(0xFF44, 0);
                self.window_line = 0;
            }

            self.line_ticks = 0;
        }
    }

    pub fn oam(&mut self, bus: &mut Interconnect) {
        if self.line_ticks >= 80 {
            change_lcd_mode(bus, LCDMode::XFer);

            self.pixel_fifo.current_state = FetchState::Tile;
            self.pixel_fifo.line_x = 0;
            self.pixel_fifo.fetch_x = 0;
            self.pixel_fifo.pushed_x = 0;
            self.pixel_fifo.fifo_x = 0;
        }

        if self.line_ticks == 1 {
            // Read OAM on the first tick only
            self.line_sprites.clear();

            self.load_line_sprites(bus);
        }
    }

    pub fn xfer(&mut self, bus: &mut Interconnect, framebuffer: &mut [u32]) {
        self.pipeline_process(bus, framebuffer);
        
        if self.pixel_fifo.pushed_x >= XRES as u8 {
            self.pipeline_reset();

            change_lcd_mode(bus, LCDMode::HBlank);

            if status_stat_int(bus, StatusSrc::HBlank) {
                bus.request_interrupt(InterruptType::LcdStat);
            }
        }
    }

    fn increment_ly(&mut self, bus: &mut Interconnect) -> u8 {
        let mut ly = lcd_read_ly(bus);
        if self.window_visible(bus) && 
            ly >= lcd_read_win_y(bus) && ly < lcd_read_win_y(bus).wrapping_add(YRES as u8) {
            self.window_line = self.window_line.wrapping_add(1);
        }
        ly = ly.wrapping_add(1);
        lcd_write_ly(bus, ly);

        if ly == bus.read(0xFF45) { // If LY == LYC
            status_lyc_set(bus, 1);

            if status_stat_int(bus, StatusSrc::LYC) {
                bus.request_interrupt(crate::interconnect::InterruptType::LcdStat);
            }
        } else {
            status_lyc_set(bus, 0);
        }
        ly
    }
}