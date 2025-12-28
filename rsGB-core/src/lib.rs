mod cart;
mod cpu;
mod debug;
mod interconnect;
mod ppu;
mod utils;

use std::path::PathBuf;

use crate::{
    cart::Cartridge, 
    cpu::CPU,  
    interconnect::Interconnect, 
    ppu::PPU,
    utils::TICKS_PER_SAMPLE,
};

pub use debug::DebugInfo;

pub use utils::{
    Button,
    ColorMode
};

struct Devices {
    bus: Interconnect,
    ppu: PPU,

    audio_callback: Box<dyn FnMut((f32, f32)) + Send>,
    framebuffer: Option<*mut [u32]>,
    pending_frame: bool,

    ticks: u64,
    last_sample_tick: u64,
}

impl Devices {
    fn new<F>(bus: Interconnect, ppu: PPU, audio_callback: F) -> Devices 
    where F: FnMut((f32, f32)) + Send + 'static {
        Devices {
            bus,
            ppu,
            audio_callback: Box::new(audio_callback),
            framebuffer: None,
            pending_frame: false,
            ticks: 0,
            last_sample_tick: 0,
        }
    }

    fn incr_cycle(&mut self, cpu_cycles: u16) {
        for _ in 0..cpu_cycles {
            for _ in 0..4 {
                self.ticks += 1;
                self.bus.tick_t();
                if let Some(ptr) = self.framebuffer {
                    unsafe {
                        let fb = &mut *ptr;
                        if self.ppu.tick(&mut self.bus, fb) { // Frame updated
                            self.pending_frame = true;
                        }
                    } 
                }

                if self.ticks - self.last_sample_tick >= TICKS_PER_SAMPLE {
                    if let Some(sample) = self.bus.apu_output() {
                        (self.audio_callback)(sample)
                    }
                    self.last_sample_tick = self.ticks;
                }
            }
            self.bus.tick_m();
        }
    }

    fn attach_buffer(&mut self, framebuffer: &mut [u32]) {
        if framebuffer.len() < 0x5A00 {
            panic!("Trying to attach framebuffer that is too small!");
        }
        self.framebuffer = Some(framebuffer as *mut [u32])
    }

    fn detach_buffer(&mut self) {
        self.framebuffer = None;
    }
}

pub struct Gameboy {
    cpu: CPU,
    devices: Devices,

    save_path: String,
}

impl Gameboy {
    pub fn new<F>(rom_path: &PathBuf, color_mode: ColorMode, audio_callback: F) -> Gameboy 
    where F: FnMut((f32, f32)) + Send + 'static {
        let save_path = format!("{}.sav", rom_path.file_stem().unwrap().to_str().unwrap()); // Here unwrap is used because we assume a correct extension is checked before

        let mut bus = Interconnect::new(color_mode);
        let ppu = PPU::new();

        let cartridge = Cartridge::load(rom_path).unwrap();
        bus.set_cart(cartridge);
        bus.load_save(&save_path);

        let devices = Devices::new(bus, ppu, audio_callback);
        Gameboy {
            cpu: CPU::new(),
            devices,

            save_path,
        }
    }

    pub fn next_frame(&mut self, framebuffer: &mut [u32]) {
        self.devices.attach_buffer(framebuffer);
        while !self.devices.pending_frame {
            self.cpu.step(&mut self.devices);
        }

        if self.devices.bus.need_save() {
            self.devices.bus.save(&self.save_path);
        }
        self.devices.pending_frame = false;
        self.devices.detach_buffer();
    }

    pub fn update_button(&mut self, button: Button, value: bool) {
        self.devices.bus.update_button(button, value);
    }

    pub fn debug(&self) -> DebugInfo {
        DebugInfo {
            cpu_registers: &self.cpu.registers,
            vram: &self.devices.bus.vram
        }
    }
}