use crate::{
    interconnect::{Interconnect, InterruptType}, ppu::utils::{lcd_read_ly, lcd_write_ly, stat_line}, 
};

use super::{
    LINES_PER_FRAME, PPU, TICKS_PER_LINE, XRES, YRES,
    utils::{LCDMode, status_lyc_set, change_lcd_mode, status_mode_set}
};

impl PPU {
    pub fn hblank(&mut self, bus: &mut Interconnect) {
        if self.line_ticks >= TICKS_PER_LINE {
            let ly = increment_ly(bus);
            self.scanline_complete();

            if ly >= YRES as u8 {
                let was_stat_line_high = stat_line(bus);
                change_lcd_mode(bus, LCDMode::VBlank);

                bus.request_interrupt(InterruptType::VBlank);

                if !was_stat_line_high && stat_line(bus) {
                    bus.request_interrupt(InterruptType::LcdStat);
                }

                self.current_frame += 1;
                self.new_frame = true;
            } else {
                let was_stat_line_high = stat_line(bus);
                status_mode_set(bus, LCDMode::OAM);
                if !was_stat_line_high && stat_line(bus) {
                    bus.request_interrupt(InterruptType::LcdStat);
                }
            }
            self.line_ticks = 0;
        }
    }

    pub fn vblank(&mut self, bus: &mut Interconnect) {
        if self.line_ticks >= TICKS_PER_LINE {
            let ly = increment_ly(bus);

            if ly >= LINES_PER_FRAME {
                self.frame_complete();

                let was_stat_line_high = stat_line(bus);

                change_lcd_mode(bus, LCDMode::OAM);
                lcd_write_ly(bus, 0);
                let ly_lyc_match = 0 == bus.read(0xFF45);
                status_lyc_set(bus, ly_lyc_match);

                if !was_stat_line_high && stat_line(bus) {
                    bus.request_interrupt(InterruptType::LcdStat);
                }
            }

            self.line_ticks = 0;
        }
    }

    pub fn oam(&mut self, bus: &mut Interconnect) {
        self.oam_fetch(bus);

        if self.line_ticks >= 80 {
            change_lcd_mode(bus, LCDMode::XFer);
            self.pipeline_reset();
            self.visible_sprites.sort_by_key(|e| e.x);
        }
    }

    pub fn xfer(&mut self, bus: &mut Interconnect, framebuffer: &mut [u32], render: bool) {
        self.process_fifo(bus, framebuffer, render);

        if self.screen_x >= XRES as u8 {
            println!("Mode 3 length: {}", self.line_ticks - 80);
            let was_stat_line_high = stat_line(bus);
            change_lcd_mode(bus, LCDMode::HBlank);

            if !was_stat_line_high && stat_line(bus) {
                bus.request_interrupt(InterruptType::LcdStat);
            }
        }
    }
}

fn increment_ly(bus: &mut Interconnect) -> u8 {
    let mut ly = lcd_read_ly(bus);

    ly = ly.wrapping_add(1);
    lcd_write_ly(bus, ly);

    let was_stat_line_high = stat_line(bus);
    let ly_lyc_match = ly == bus.read(0xFF45);
    status_lyc_set(bus, ly_lyc_match);

    if !was_stat_line_high && stat_line(bus) {
        bus.request_interrupt(InterruptType::LcdStat);
    }

    ly
}