mod cart;
mod cpu;
mod utils;

use std::{sync::RwLock, thread, time::Duration};

use cart::Cartridge;
use cpu::CPU;

use minifb;

/* 
    Emu components :

    |Cart|
    |CPU|
    |Address Bus|
    |PPU|
    |Timer|

*/

pub struct EmuContext {
    paused: bool,
    running: bool,
    ticks: u64,
}

static EMU_CONTEXT: RwLock<EmuContext> = RwLock::new(EmuContext {
    paused: false,
    running: false,
    ticks: 0,
});

pub fn emu_run(args: Vec<String>) {
    if args.len() < 2 {
        println!("Usage: rsgb <rom_file>");
        return
    }

    let cart = match Cartridge::load(&args[1]) {
        Ok(cart) => cart,
        Err(e) => {
            eprintln!("Failed to load the ROM file: {}", args[1]);
            eprintln!("Error: {e}");
            return
        }
    };

    println!("Cart loaded...");

    // Framebuffer init

    let cpu = CPU::new();

    let mut ctx = EMU_CONTEXT.write()
        .expect("The context has been poisoned");

    ctx.running = true;
    ctx.paused = false;
    ctx.ticks = 0;

    while ctx.running {
        if ctx.paused {
            thread::sleep(Duration::from_millis(10));
            continue
        }

        if !cpu.step() {
            println!("CPU Stopped");
            return
        }
    }
}