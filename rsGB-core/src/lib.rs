mod cart;
mod cpu;
mod interconnect;
mod ppu;
mod utils;
mod dbg;

mod communicators;

use std::{
    sync::{Arc, Mutex, MutexGuard}, 
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

pub use communicators::*;

// use minifb;

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

struct Emulator {
    cpu: CPU,
    devices: Devices,

    save_path: String,
}

impl Emulator {
    fn new(ctx: &mut MutexGuard<'_, EmuContext>) -> Emulator {
        let save_path = ctx.file_path.clone().replace(".gb", ".sav");
        let devices = ctx.emulator_devices.take().unwrap();
        
        Emulator {
            cpu: CPU::new(),
            devices,

            save_path,
        }
    }

    fn load_cart(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cartridge = Cartridge::load(path)?;
        self.devices.bus.set_cart(cartridge);
        self.devices.bus.load(&self.save_path);
        Ok(())
    }

    fn step(&mut self) -> bool {
        let res = self.cpu.step(&mut self.devices);

        if self.devices.ppu.send_new_frame() {
            if self.need_save() {
                self.save();
            }
        }
        res
    }

    fn need_save(&self) -> bool {
        self.devices.bus.need_save()
    }

    fn save(&mut self) {
        self.devices.bus.save(&self.save_path);
    }
}

pub fn run(context: Arc<Mutex<EmuContext>>) {
    let mut ctx: MutexGuard<'_, EmuContext> = context.lock().unwrap();

    let mut emulator = Emulator::new(&mut ctx);

    emulator.load_cart(&ctx.file_path).expect("Failed to load the cartridge");

    // println!("Cart loaded...");

    start_emulation(ctx);

    loop {
        let ctx = context.lock().unwrap();
        if !ctx.is_running() {
            break
        }

        if ctx.is_paused() {
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        if !emulator.step() {
            println!("CPU Stopped");
            break;
        }
    }
}

fn start_emulation(mut ctx: MutexGuard<'_, EmuContext>) {
    ctx.start();
}