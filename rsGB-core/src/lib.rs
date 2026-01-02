mod cart;
mod cpu;
mod debug;
mod interconnect;
mod ppu;
mod utils;
pub mod settings;

use std::path::{Path, PathBuf};

use crate::{
    cart::Cartridge, cpu::CPU, interconnect::Interconnect, ppu::PPU, settings::SaveLocation, utils::TICKS_PER_SAMPLE
};

pub use debug::DebugInfo;

pub use utils::{
    Button, InputState,
    ColorMode
};

use settings::{
    Settings,
};

struct Devices {
    bus: Interconnect,
    ppu: PPU,

    audio_callback: Box<dyn FnMut((f32, f32)) + Send>,
    framebuffer: Option<*mut [u32]>,

    speed: u8,
    frames: u8,

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

            speed: 1,
            frames: 0,

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
                        if self.ppu.tick(&mut self.bus, fb, self.frames == self.speed - 1) { // Frame updated
                            self.frames += 1;
                        }
                    } 
                }

                if self.ticks - self.last_sample_tick >= TICKS_PER_SAMPLE * self.speed as u64 {
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

    save_path: PathBuf,
}

impl Gameboy {
    pub fn new<F>(color_mode: ColorMode, audio_callback: F) -> Gameboy 
    where F: FnMut((f32, f32)) + Send + 'static {
        let bus = Interconnect::new(color_mode);
        let ppu = PPU::new();

        let devices = Devices::new(bus, ppu, audio_callback);
        Gameboy {
            cpu: CPU::new(),
            devices,

            save_path: PathBuf::new(),
        }
    }

    pub fn load_cartridge(&mut self, rom_path: &PathBuf, settings: &Settings) {
        let save_path = match settings.get_save_location() {
            SaveLocation::GameLoc => {
                let mut clone = rom_path.clone();
                clone.set_extension("sav");
                clone
            },
            SaveLocation::SaveFolder(path) => {
                let file_name = Path::new(self.save_path.file_name().unwrap());
                
                let mut clone = path.clone();
                clone.push(file_name);

                clone
            }
        };

        let cartridge = Cartridge::load(rom_path).unwrap();
        self.devices.bus.set_cart(cartridge);
        self.devices.bus.load_save(&save_path);

        self.save_path = save_path;
    }

    pub fn next_frame(&mut self, framebuffer: &mut [u32], settings: &Settings) {
        self.devices.attach_buffer(framebuffer);

        let speed = settings.speed as u8;
        self.devices.speed = speed;

        while self.devices.frames < speed {
            self.cpu.step(&mut self.devices);
        }
        
        if self.devices.bus.need_save() {
            self.devices.bus.save(&self.save_path);
        }
        self.devices.frames = 0;
        self.devices.detach_buffer();
    }

    pub fn apply_input(&mut self, input: InputState) {
        self.devices.bus.update_input(input);
    }

    pub fn cartridge_loaded(&self) -> bool {
        self.devices.bus.cart.is_some()
    }

    pub fn debug(&self) -> DebugInfo {
        let vram_updated = self.devices.bus.vram_updated.get();
        self.devices.bus.vram_updated.replace(false);

        DebugInfo::new(
            &self.cpu, 
            vram_updated,
            &self.devices.bus.vram, 
            &self.devices.bus.cart.as_ref().unwrap()
        )
    }
}

