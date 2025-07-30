mod cart;
mod cpu;
mod interconnect;
mod ppu;
mod timer;
mod utils;
mod dbg;

use std::{sync::{Arc, Mutex, MutexGuard}, thread, time::Duration};

use crate::{
    cart::Cartridge, 
    cpu::CPU, 
    ppu::PPU,
    interconnect::Interconnect,
    dbg::Debugger,
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

pub struct EmuContext {
    file_path: String,

    debug: bool,
    paused: bool,
    running: bool,
    ticks: u64,

    debugger: Option<Debugger>
}

impl EmuContext {
    pub fn new(path: &str, debug: bool) -> EmuContext {
        EmuContext {
            file_path: path.to_string(),

            debug,
            paused: false,
            running: false,
            ticks: 0,

            debugger: if debug { Some(Debugger::new()) } else { None },
        }
    }

    fn incr_cycle(&mut self) {
        self.ticks += 1;
    }

    fn start(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
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
    cpu: CPU,
    ppu: PPU,
}

impl Emulator {
    fn new() -> Emulator {
        Emulator {
            bus: Interconnect::new(),
            cpu: CPU::new(),
            ppu: PPU::new(),
        }
    }

    fn load_cart(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cartridge = Cartridge::load(path)?;
        self.bus.set_cart(cartridge);
        Ok(())
    }

    fn step(&mut self, ctx: &mut MutexGuard<'_, EmuContext>) -> bool {
        self.cpu.step(&mut self.bus, ctx)
    }
}

pub fn run(context: Arc<Mutex<EmuContext>>) {
    let ctx: MutexGuard<'_, EmuContext> = context.lock().unwrap();
    let mut emulator = Emulator::new();

    emulator.load_cart(&ctx.file_path)
        .expect(&format!("Failed to load the ROM file: {}", &ctx.file_path));

    // println!("Cart loaded...");

    start_emulation(ctx);

    loop {
        let mut ctx = context.lock().unwrap();
        if !ctx.is_running() {
            break
        }

        if ctx.is_paused() {
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        if !emulator.step(&mut ctx) {
            println!("CPU Stopped");
            break;
        }
    }
}

fn start_emulation(mut ctx: MutexGuard<'_, EmuContext>) {
    ctx.start();
}