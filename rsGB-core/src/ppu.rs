use crate::{interconnect::Interconnect, Devices};

mod state_machine;

const LINES_PER_FRAME: u8 = 154;
const TICKS_PER_LINE: u32 = 456;
const YRES: usize = 144;
const XRES: usize = 160;

pub struct PPU {
    current_frame: u32,
    line_ticks: u32,
    video_buffer: [u32; XRES * YRES],
    new_frame: bool,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            current_frame: 0,
            line_ticks: 0,
            video_buffer: [0; XRES* YRES],
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

    pub fn oam_read(&self, dev: &mut Devices, address: u16) -> u8 {
        dev.bus_read(address)
    }

    pub fn oam_write(&self, dev: &mut Devices, address: u16, value: u8) {
        dev.bus_write(address, value);
    }

    pub fn vram_read(&self, dev: &mut Devices, address: u16) -> u8 {
        dev.bus_read(address)
    }

    pub fn vram_write(&self, dev: &mut Devices, address: u16, value: u8) {
        dev.bus_write(address, value);
    }

    pub fn get_frame(&self) -> Option<[u32; XRES * YRES]> {
        if self.new_frame {
            Some(self.video_buffer)
        } else {
            None
        }
    }
}
