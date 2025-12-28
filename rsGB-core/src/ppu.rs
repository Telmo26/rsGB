use std::collections::VecDeque;

use crate::interconnect::{Interconnect, OAMEntry};

mod state_machine;
mod pipeline;
mod utils;
mod fetcher;

use fetcher::Fetcher;
use utils::{status_mode, LCDMode};

const LINES_PER_FRAME: u8 = 154;
const TICKS_PER_LINE: u32 = 456;
const YRES: usize = 144;
const XRES: usize = 160;

#[derive(Debug)]
pub struct PPU {
    fetcher: Fetcher,
    bgw_fifo: VecDeque<(u32, u8)>,
    obj_fifo: VecDeque<(u32, u8, bool)>,

    visible_sprites: Vec<OAMEntry>,
    fetched_sprites: [bool; 10],

    pushed_x: u8, // The pixel position to push in the framebuffer
    current_x: u8, // The current position we're dealing with on the screen

    current_frame: u32,
    line_ticks: u32,
    new_frame: bool,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            fetcher: Fetcher::new(),
            bgw_fifo: VecDeque::with_capacity(8),
            obj_fifo: VecDeque::with_capacity(8),

            visible_sprites: Vec::with_capacity(10),
            fetched_sprites: [false; 10],

            pushed_x: 0,
            current_x: 0,

            current_frame: 0,
            line_ticks: 0,
            new_frame: false,
        }
    }

    pub fn tick(&mut self, bus: &mut Interconnect, framebuffer: &mut [u32], render: bool) -> bool {
        self.line_ticks += 1;

        let lcd_mode = status_mode(bus);

        match lcd_mode {
            LCDMode::HBlank => self.hblank(bus),
            LCDMode::VBlank => self.vblank(bus),
            LCDMode::OAM => self.oam(bus),
            LCDMode::XFer => self.xfer(bus, framebuffer, render),
        };

        if self.new_frame {
            self.new_frame = false;
            true
        } else {
            false
        }
    }
}