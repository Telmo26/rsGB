mod cart;
mod cpu;
mod debug;
mod interconnect;
mod ppu;
mod utils;
pub mod settings;

use std::path::{Path, PathBuf};

use crate::{
    cart::Cartridge, cpu::CPU, interconnect::Interconnect, ppu::PPU, settings::SaveLocation, utils::{AUDIO_FREQUENCY, CPU_FREQUENCY}
};

pub use debug::DebugInfo;

pub use utils::{
    Button, InputState,
    ColorMode
};

use settings::{
    Settings,
};

pub trait VideoSink {
    /// Gives mutable access to the framebuffer currently being rendered into
    fn get_mut(&mut self) -> &mut [u32];

    /// Called once when the frame rendering is done
    fn present(&mut self);
}

pub trait AudioSink {
    /// Used to send a sample to the playing thread
    fn push_sample(&mut self, left: f32, right: f32);
}

/// This trait is used to abstract away the various peripherals and only expose
/// the required reading and writing functions
trait Peripherals {
    fn incr_cycle(&mut self);
    
    fn read8(&self, address: u16) -> u8;
    fn write8(&mut self, address: u16, value: u8);
    fn write16(&mut self, address: u16, value: u16);
    fn ie_register(&self) -> u8;
}

struct Devices<A: AudioSink, V: VideoSink> {
    bus: Interconnect,
    ppu: PPU,

    audio_sink: A,
    video_sink: V,

    speed: u8,
    frames: u8,

    ticks: u64,
    audio_accumulator: u32,
}

impl<A, V> Devices<A, V>
where 
    A: AudioSink,
    V: VideoSink
{
    fn new(
        bus: Interconnect, 
        ppu: PPU, 
        audio_sink: A,
        video_sink: V
    ) -> Devices<A, V>
    {
        Devices {
            bus,
            ppu,
            audio_sink,
            video_sink,

            speed: 1,
            frames: 0,

            ticks: 0,
            audio_accumulator: 0,
        }
    }
}

impl<A, V> Peripherals for Devices<A, V>
where 
    A: AudioSink,
    V: VideoSink,
{
    fn incr_cycle(&mut self) {
        for _ in 0..4 {
            self.ticks += 1;
            self.bus.tick_t();

            let framebuffer = self.video_sink.get_mut();

            if self.ppu.tick(&mut self.bus, framebuffer, self.frames == self.speed - 1) { // Frame updated
                self.frames += 1;
            }

            self.audio_accumulator += AUDIO_FREQUENCY;
            if self.audio_accumulator >= CPU_FREQUENCY * self.speed as u32 {
                self.audio_accumulator -= CPU_FREQUENCY * self.speed as u32;
                if let Some((left, right)) = self.bus.apu_output() {
                    self.audio_sink.push_sample(left, right);
                }
            }
        }
        self.bus.tick_m();
    }

    fn read8(&self, address: u16) -> u8 {
        self.bus.read(address)
    }

    fn write8(&mut self, address: u16, value: u8) {
        self.bus.write(address, value);
    }

    fn write16(&mut self, address: u16, value: u16) {
        self.bus.write16(address, value);
    }

    fn ie_register(&self) -> u8 {
        self.bus.get_ie_register()
    }
}

pub struct Gameboy<A: AudioSink, V: VideoSink> {
    cpu: CPU,
    devices: Devices<A, V>,

    save_path: PathBuf,
}

impl<A, V> Gameboy<A, V> 
where 
    A: AudioSink,
    V: VideoSink    
{
    pub fn new(
        color_mode: ColorMode, 
        audio_sink: A,
        video_sink: V
    ) -> Gameboy<A, V> 
    {
        let bus = Interconnect::new(color_mode);
        let ppu = PPU::new();

        let devices = Devices::new(bus, ppu, audio_sink, video_sink);
        Gameboy {
            cpu: CPU::new(),
            devices,

            save_path: PathBuf::new(),
        }
    }

    pub fn load_cartridge(&mut self, rom_path: &Path, settings: &Settings) {
        let save_path = match settings.get_save_location() {
            SaveLocation::GameLoc => {
                let mut clone = rom_path.to_path_buf();
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

    pub fn next_frame(&mut self, settings: &Settings) {
        let speed = settings.speed as u8;
        self.devices.speed = speed;

        while self.devices.frames < speed {
            self.cpu.step(&mut self.devices);
        }
        
        if self.devices.bus.need_save() {
            self.devices.bus.save(&self.save_path);
        }
        self.devices.frames = 0;
    }

    pub fn apply_input(&mut self, input: InputState) {
        self.devices.bus.update_input(input);
    }

    pub fn cartridge_loaded(&self) -> bool {
        self.devices.bus.cart.is_some()
    }

    pub fn debug<'a>(&'a self) -> DebugInfo<'a> {
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