mod cart;
mod cpu;
mod interconnect;
mod ppu;
mod utils;
mod dbg;

use std::{
    sync::{Arc, Condvar, Mutex, MutexGuard}, 
    thread, 
    time::Duration
};

use crate::{
    cart::Cartridge, 
    cpu::CPU, 
    ppu::PPU,
    interconnect::Interconnect,
    dbg::Debugger,
};

use ringbuf::{traits::Split, HeapCons};

pub type Frame = [u32; 0x5A00];
pub type VRAM = [u8; 0x2000];

pub enum Button {
    A,
    B,
    START,
    SELECT,
    UP,
    DOWN,
    LEFT,
    RIGHT
}

/*
    Emu components :

    |Cart|
    |CPU|
    |Address Bus|
    |PPU|
    |Timer|

*/

struct Devices {
    bus: Interconnect,
    ppu: PPU,

    debugger: Option<Debugger>,
    ticks: u64,
}

impl Devices {    
    fn incr_cycle(&mut self, cpu_cycles: u16) {
        for _ in 0..cpu_cycles {
            for _ in 0..4 {
                self.ticks += 1;
                self.bus.tick_t();
                self.ppu.tick(&mut self.bus);
            }
            self.bus.tick_m();
        } 
    }
}

pub struct Gameboy {
    cpu: CPU,
    devices: Devices,

    audio_receiver: Option<HeapCons<(f32, f32)>>,

    save_path: String,
}

impl Gameboy {
    pub fn new(rom_path: &str, debug: bool) -> Gameboy {
        let save_path = rom_path.replace(".gb", ".sav");

        let rb = ringbuf::HeapRb::<(f32, f32)>::new(8192);
        let (audio_sender, audio_receiver) = rb.split();

        let mut bus = Interconnect::new(audio_sender);
        let ppu = PPU::new();

        let cartridge = Cartridge::load(rom_path).unwrap();
        bus.set_cart(cartridge);
        bus.load(&save_path);

        let devices = Devices {
            bus,
            ppu,
            debugger: if debug { Some(Debugger::new()) } else { None },

            ticks: 0,
        };
        
        Gameboy {
            cpu: CPU::new(),
            devices,

            audio_receiver: Some(audio_receiver),
            save_path,
        }
    }

    pub fn next_frame(&mut self) -> &Frame {
        while !self.devices.ppu.is_new_frame() {
            self.cpu.step(&mut self.devices);
        }
        if self.devices.bus.need_save() {
            self.devices.bus.save(&self.save_path);
        }
        let frame = self.devices.ppu.get_frame();
        frame.unwrap()
    }

    pub fn update_button(&mut self, button: Button, value: bool) {
        self.devices.bus.update_button(button, value);        
    }

    pub fn audio_receiver(&mut self) -> HeapCons<(f32, f32)> {
        self.audio_receiver.take().unwrap()
    }
}