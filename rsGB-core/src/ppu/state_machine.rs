use crate::{
    interconnect::{Interconnect, InterruptType}, ppu::utils::{lcd_read_ly, lcd_write_ly}, 
};

use super::{
    LINES_PER_FRAME, PPU, TICKS_PER_LINE, XRES, YRES,
    utils::{LCDMode, StatusSrc, status_stat_int, status_lyc_set, change_lcd_mode, status_mode_set}
};

impl PPU {
    pub fn hblank(&mut self, bus: &mut Interconnect) {
        if self.line_ticks >= TICKS_PER_LINE {
            let ly = increment_ly(bus);
            self.scanline_complete();

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
            let ly = increment_ly(bus);

            if ly >= LINES_PER_FRAME {
                self.frame_complete();
                change_lcd_mode(bus, LCDMode::OAM);
                bus.write(0xFF44, 0);
            }

            self.line_ticks = 0;
        }
    }

    pub fn oam(&mut self, bus: &mut Interconnect) {
        if self.line_ticks >= 80 {
            change_lcd_mode(bus, LCDMode::XFer);
            self.pipeline_reset();
        } else {
            self.oam_fetch(bus);
        }
    }

    pub fn xfer(&mut self, bus: &mut Interconnect, framebuffer: &mut [u32]) {
        self.process_fifo(bus, framebuffer);

        if self.pushed_x >= XRES as u8 {
            change_lcd_mode(bus, LCDMode::HBlank);

            if status_stat_int(bus, StatusSrc::HBlank) {
                bus.request_interrupt(InterruptType::LcdStat);
            }
        }
    }
}

fn increment_ly(bus: &mut Interconnect) -> u8 {
    let mut ly = lcd_read_ly(bus);

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