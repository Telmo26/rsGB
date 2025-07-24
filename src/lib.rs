mod cart;
mod cpu;
mod interconnect;
mod ppu;
mod timer;
mod utils;

use std::{sync::{Arc, Mutex, MutexGuard}, thread, time::Duration};

use crate::{
    cart::Cartridge, 
    cpu::Cpu, 
    ppu::Ppu,
    interconnect::Interconnect,
};

// use minifb;

/*
    Emu components :

    |Cart|
    |CPU|
    |Address Bus|
    |PPU|
    |Timer|

*/

struct EmuContext {
    paused: bool,
    running: bool,
    ticks: u64,
}

impl EmuContext {
    fn new() -> EmuContext {
        EmuContext {
            paused: false,
            running: false,
            ticks: 0
        }
    }

    fn incr_cycle(&mut self, cycles: u64) {
        self.ticks += cycles;
    }

    fn start(&mut self) {
        self.running = true;
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn is_paused(&self) -> bool {
        self.paused
    }
}

struct Emulator {
    bus: Interconnect,
    cpu: Cpu,
    ppu: Ppu,
}

impl Emulator {
    fn new() -> Emulator {
        Emulator {
            bus: Interconnect::new(),
            cpu: Cpu::new(),
            ppu: Ppu::new(),
        }
    }

    fn load_cart(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cartridge = Cartridge::load(path)?;
        self.bus.set_cart(cartridge);
        Ok(())
    }

    fn step(&mut self, mut ctx: MutexGuard<'_, EmuContext>) -> bool {
        self.cpu.step(&mut self.bus, &mut ctx)
    }
}

pub fn run(args: Vec<String>) {
    if args.len() < 2 {
        println!("Usage: rsgb <rom_file>");
        return;
    }

    let ctx = Arc::new(Mutex::new(EmuContext::new()));

    let mut emulator = Emulator::new();

    emulator.load_cart(&args[1])
        .expect(&format!("Failed to load the ROM file: {}", args[1]));

    println!("Cart loaded...");

    ctx.lock().unwrap() // This should never fail, it is the first call
        .start();

    loop {
        let context = ctx.lock().unwrap();
        if !context.is_running() {
            break
        }

        if context.is_paused() {
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        if !emulator.step(context) {
            println!("CPU Stopped");
            break;
        }
    }
}
