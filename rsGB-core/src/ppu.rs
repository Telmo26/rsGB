use crate::{interconnect::{Interconnect, InterruptType, OAMEntry}, ppu::utils::{stat_line, status_mode_set}, utils::BoundedQueue};

mod state_machine;
mod pipeline;
mod utils;
mod fetcher;

use fetcher::Fetcher;
pub use utils::LCDMode;

const LINES_PER_FRAME: u8 = 154;
const TICKS_PER_LINE: u32 = 456;
const OAM_DURATION: u32 = 80;
const YRES: usize = 144;
const XRES: usize = 160;

#[derive(Debug)]
pub struct PPU {
    fetcher: Fetcher,
    bgw_fifo: BoundedQueue<(u32, u8), 16>,
    obj_fifo: BoundedQueue<(u32, u8, bool), 16>,

    visible_sprites: Vec<OAMEntry>,
    fetched_sprites: [bool; 10],

    screen_x: u8, // The pixel position to push in the framebuffer
    current_x: u8, // The current position we're dealing with on the screen

    
    stat_delay: u8,
    stat_line: bool,
    stat_mode: LCDMode,

    current_frame: u32,
    line_ticks: u32,
    new_frame: bool,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            fetcher: Fetcher::new(),
            bgw_fifo: BoundedQueue::default(),
            obj_fifo: BoundedQueue::default(),

            visible_sprites: Vec::with_capacity(10),
            fetched_sprites: [false; 10],

            screen_x: 0,
            current_x: 0,

            stat_delay: 0,
            stat_line: false,
            stat_mode: LCDMode::HBlank,

            current_frame: 0,
            line_ticks: 0,
            new_frame: false,
        }
    }

    pub fn tick(&mut self, bus: &mut Interconnect, framebuffer: &mut [u32], render: bool) -> bool {
        if self.stat_delay > 0 {
            self.stat_delay -= 1;

            if self.stat_delay == 0 {
                status_mode_set(bus, self.stat_mode);

                let new_line = stat_line(bus);

                if new_line && !self.stat_line {
                    bus.request_interrupt(InterruptType::LcdStat);
                }

                self.stat_line = new_line;
            }
        }
        
        match self.stat_mode {
            LCDMode::HBlank => self.hblank(bus),
            LCDMode::VBlank => self.vblank(bus),
            LCDMode::OAM => self.oam(bus),
            LCDMode::XFer => self.xfer(bus, framebuffer, render),
        };

        self.line_ticks = (self.line_ticks + 1) % TICKS_PER_LINE;

        if self.new_frame {
            self.new_frame = false;
            true
        } else {
            false
        }
    }
}