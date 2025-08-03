use crate::{
    interconnect::{Interconnect, InterruptType}, 
};

use super::{
    LINES_PER_FRAME, PPU, TICKS_PER_LINE, XRES, YRES,
    pixel_fifo::FetchState,
    utils::{LCDMode, StatusSrc, status_stat_int, status_lyc_set, change_lcd_mode, status_mode_set}
};

const TARGET_FRAME_TIME: f64 = 1000.0 / 60.0;
const PREV_FRAME_TIME: f64 = 0.0;
const START_TIMER: f64 = 0.0;
const FRAME_COUNT: f64 = 0.0;

impl PPU {
    pub fn hblank(&mut self, bus: &mut Interconnect) {
        if self.line_ticks >= TICKS_PER_LINE {
            let mut ly = bus.read(0xFF44);
            increment_ly(bus, &mut ly);

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
            let mut ly = bus.read(0xFF44);
            increment_ly(bus, &mut ly);

            if ly >= LINES_PER_FRAME {
                change_lcd_mode(bus, LCDMode::OAM);
                bus.write(0xFF44, 0);
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
    }

    pub fn xfer(&mut self, bus: &mut Interconnect) {
        self.pixel_fifo.process(bus, &mut self.video_buffer, self.line_ticks);
        
        if self.pixel_fifo.pushed_x >= XRES as u8 {
            self.pixel_fifo.reset();

            change_lcd_mode(bus, LCDMode::HBlank);

            if status_stat_int(bus, StatusSrc::HBlank) {
                bus.request_interrupt(InterruptType::LcdStat);
            }
        }
    }
}

fn increment_ly(bus: &mut Interconnect, ly: &mut u8) {
    *ly = ly.wrapping_add(1);
    bus.write(0xFF44, *ly);

    if *ly == bus.read(0xFF45) { // If LY == LYC
        status_lyc_set(bus, 1);

        if status_stat_int(bus, StatusSrc::LYC) {
            bus.request_interrupt(crate::interconnect::InterruptType::LcdStat);
        }
    } else {
        status_lyc_set(bus, 0);
    }
}