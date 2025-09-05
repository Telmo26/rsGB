use crate::{interconnect::{Interconnect, OAMEntry}, ppu::utils::{status_mode, LCDMode}};

mod state_machine;
mod pipeline;
mod utils;

use pipeline::PixelFifo;

const LINES_PER_FRAME: u8 = 154;
const TICKS_PER_LINE: u32 = 456;
const YRES: usize = 144;
const XRES: usize = 160;

pub struct PPU {
    line_sprites: Vec<OAMEntry>, // Capacity: 10

    fetched_entries: Vec<OAMEntry>, // Capacity: 3

    window_line: u8,

    pixel_fifo: PixelFifo,
    
    current_frame: u32,
    line_ticks: u32,
    new_frame: bool,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            line_sprites: Vec::with_capacity(10),

            fetched_entries: Vec::with_capacity(3),

            window_line: 0,

            pixel_fifo: PixelFifo::new(),

            current_frame: 0,
            line_ticks: 0,
            new_frame: false,
        }
    }

    pub fn tick(&mut self, bus: &mut Interconnect, framebuffer: &mut [u32]) -> bool {
        self.line_ticks += 1;

        let lcd_mode = status_mode(bus);

        match lcd_mode {
            LCDMode::HBlank => self.hblank(bus),
            LCDMode::VBlank => self.vblank(bus),
            LCDMode::OAM => self.oam(bus),
            LCDMode::XFer => self.xfer(bus, framebuffer),
        };

        if self.new_frame {
            self.new_frame = false;
            true
        } else {
            false
        }
    }
}