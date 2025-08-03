use std::collections::VecDeque;

use crate::{Devices, interconnect::Interconnect};

mod state_machine;
mod pixel_fifo;
mod utils;

use pixel_fifo::PixelFifo;

const LINES_PER_FRAME: u8 = 154;
const TICKS_PER_LINE: u32 = 456;
const YRES: usize = 144;
const XRES: usize = 160;
const PIXELS: usize = 0x5A00;

pub struct PPU {
    pixel_fifo: PixelFifo,
    current_frame: u32,
    line_ticks: u32,
    video_buffer: [u32; PIXELS],
    new_frame: bool,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            pixel_fifo: PixelFifo::new(),
            current_frame: 0,
            line_ticks: 0,
            video_buffer: [0; PIXELS],
            new_frame: false,
        }
    }

    pub fn tick(&mut self, bus: &mut Interconnect) {
        self.line_ticks += 1;
        self.new_frame = false;

        let lcd_status = bus.read(0xFF41);
        let mode = lcd_status & 0b11;

        match mode {
            0 => self.hblank(bus),
            1 => self.vblank(bus),
            2 => self.oam(bus),
            3 => self.xfer(bus),
            _ => panic!(),
        }
    }

    pub fn oam_read(&self, dev: &Devices, address: u16) -> u8 {
        dev.bus.read(address)
    }

    pub fn oam_write(&self, dev: &mut Devices, address: u16, value: u8) {
        dev.bus.write(address, value);
    }

    pub fn vram_read(&self, dev: &Devices, address: u16) -> u8 {
        dev.bus.read(address)
    }

    pub fn vram_write(&self, dev: &mut Devices, address: u16, value: u8) {
        dev.bus.write(address, value);
    }

    pub fn get_frame(&self) -> Option<[u32; XRES * YRES]> {
        if self.new_frame {
            Some(self.video_buffer)
        } else {
            None
        }
    }
}
