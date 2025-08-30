use std::sync::{Arc, Condvar, Mutex};

use crate::{interconnect::{Interconnect, OAMEntry}, ppu::utils::{status_mode, LCDMode}, Frame};

mod state_machine;
mod pipeline;
mod utils;

use pipeline::PixelFifo;

const LINES_PER_FRAME: u8 = 154;
const TICKS_PER_LINE: u32 = 456;
const YRES: usize = 144;
const XRES: usize = 160;
const PIXELS: usize = 0x5A00;

pub struct PPU {
    line_sprites: Vec<OAMEntry>, // Capacity: 10

    fetched_entries: Vec<OAMEntry>, // Capacity: 3

    window_line: u8,

    pixel_fifo: PixelFifo,
    
    current_frame: u32,
    line_ticks: u32,

    framebuffer: [u32; PIXELS],
    frame_available: Arc<(Mutex<bool>, Condvar)>,
    new_frame: bool,

    threading_enabled: bool
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

            framebuffer: [0; PIXELS],
            frame_available: Arc::new((Mutex::new(false), Condvar::new())),
            new_frame: false,

            threading_enabled: false,
        }
    }

    pub fn tick(&mut self, bus: &mut Interconnect) {
        self.line_ticks += 1;
        self.new_frame = false;

        let lcd_mode = status_mode(bus);

        match lcd_mode {
            LCDMode::HBlank => self.hblank(bus),
            LCDMode::VBlank => self.vblank(bus),
            LCDMode::OAM => self.oam(bus),
            LCDMode::XFer => self.xfer(bus),
        }
    }

    pub fn is_new_frame(&self) -> bool {
        self.new_frame
    }

    pub fn get_frame(&mut self) -> Option<&Frame> {
        if !self.new_frame {
            return None
        } 

        self.new_frame = false;

        // if self.threading_enabled {
        //     let (lock, cvar) = &*self.frame_available;
        //     let mut frame_ready = lock.lock().unwrap();

        //     while *frame_ready {
        //         frame_ready = cvar.wait(frame_ready).unwrap();
        //     }

        //     *frame_ready = true;
        //     cvar.notify_one();
        // }

        Some(&self.framebuffer)
    }

    pub fn enable_threading(&mut self) { //-> (Arc<Mutex<Frame>>, Arc<(Mutex<bool>, Condvar)>) {
        self.threading_enabled = true;
        // (self.framebuffer.clone(), self.frame_available.clone())
    }
}
