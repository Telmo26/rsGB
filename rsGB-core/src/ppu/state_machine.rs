use crate::{
    interconnect::{Interconnect, InterruptType}, ppu::{OAM_DURATION, utils::{lcd_read_ly, lcd_write_ly, stat_line}}, 
};

use super::{
    LINES_PER_FRAME, PPU, TICKS_PER_LINE, XRES, YRES,
    utils::{LCDMode, status_lyc_set}
};

const STAT_DELAY: u8 = 4;

impl PPU {
    pub fn hblank(&mut self, bus: &mut Interconnect) {
        if self.line_ticks == TICKS_PER_LINE - 1 {
            // println!("LY increase on dot {}", self.line_ticks);
            let ly = self.increment_ly(bus);
            self.scanline_complete();

            if ly == YRES as u8 {
                self.stat_line = stat_line(bus);
                self.stat_mode = LCDMode::VBlank;
                bus.request_interrupt(InterruptType::VBlank);

                self.stat_delay = STAT_DELAY;
            } else {
                self.stat_line = stat_line(bus);
                self.stat_mode = LCDMode::OAM;
                self.stat_delay = STAT_DELAY;
            }
        }
    }

    pub fn vblank(&mut self, bus: &mut Interconnect) {
        if self.line_ticks == TICKS_PER_LINE - 1 {
            let ly = self.increment_ly(bus);

            if ly == LINES_PER_FRAME {
                self.frame_complete();

                self.stat_line = stat_line(bus);
                self.stat_mode = LCDMode::OAM;
                lcd_write_ly(bus, 0);

                let ly_lyc_match = 0 == bus.ppu_read(0xFF45);
                status_lyc_set(bus, ly_lyc_match);

                self.stat_delay = STAT_DELAY;
                
            }
        }
    }

    pub fn oam(&mut self, bus: &mut Interconnect) {
        self.oam_fetch(bus);

        if self.line_ticks == OAM_DURATION - 1 {
            self.stat_line = stat_line(bus);
            self.stat_mode = LCDMode::XFer;
            self.stat_delay = STAT_DELAY;
            
            self.pipeline_reset();
            self.visible_sprites.sort_by_key(|e| e.x);
        }
    }

    pub fn xfer(&mut self, bus: &mut Interconnect, framebuffer: &mut [u32], render: bool) {
        self.process_fifo(bus, framebuffer, render);

        if self.screen_x == XRES as u8 {   
            self.stat_line = stat_line(bus);
            self.stat_mode = LCDMode::HBlank;
            self.stat_delay = STAT_DELAY;
        }
    }

    fn increment_ly(&mut self, bus: &mut Interconnect) -> u8 {
        let mut ly = lcd_read_ly(bus);

        ly = ly.wrapping_add(1);
        lcd_write_ly(bus, ly);
        
        self.stat_line = stat_line(bus);
        // let was_stat_line_high = stat_line(bus);
        
        let ly_lyc_match = ly == bus.ppu_read(0xFF45);
        status_lyc_set(bus, ly_lyc_match);

        // if !was_stat_line_high && stat_line(bus) {
        //     bus.request_interrupt(InterruptType::LcdStat);
        // }
        self.stat_delay = STAT_DELAY;

        ly
    }
}

