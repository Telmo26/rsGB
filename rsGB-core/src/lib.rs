mod cart;
mod cpu;
mod interconnect;
mod ppu;
mod utils;
mod dbg;

mod communicators;

use std::{
    sync::{mpsc::{self, Sender}, Arc, Mutex, MutexGuard}, 
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
    fn new(gamepad_state: Arc<Mutex<GamepadState>>, debug: bool) -> Devices {
        Devices {
            bus: Interconnect::new(gamepad_state),
            ppu: PPU::new(),

            debugger: if debug { Some(Debugger::new()) } else { None },
            ticks: 0,
        }
    }
    
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

    frame_tx: FrameSender,
    debug_tx: Option<DebugSender>,
}

impl Emulator {
    fn new(file_path: String, frame_tx: FrameSender, debug_tx: Option<DebugSender>, gamepad_state: Arc<Mutex<GamepadState>>) -> Emulator {
        let devices: Devices = Devices::new(gamepad_state, debug_tx.is_some());

        Emulator {
            cpu: CPU::new(),
            devices,

            save_path: file_path.replace(".gb", ".sav"),

            frame_tx,
            debug_tx,
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

        if let Some(frame) = self.devices.ppu.get_frame() {
            // This is piloted by the UI polling rate
            self.frame_tx.send(frame).unwrap(); 
            if self.need_save() {
                self.save()
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

    fn check_debug(&mut self) {
        if self.devices.bus.vram_update {
            if let Some(tx) = &self.debug_tx {
                let _ = tx.try_send(self.devices.bus.vram[0..0x1800].try_into().unwrap());
                self.devices.bus.vram_update = false;
            }
        }
    }
}

pub fn run(context: Arc<Mutex<EmuContext>>) {
    let mut ctx: MutexGuard<'_, EmuContext> = context.lock().unwrap();
    
    let gamepad_state = ctx.gamepad_state.take().unwrap();
    let debug_tx = ctx.debug_tx.take();
    let frame_tx = ctx.frame_tx.take().unwrap();

    let mut emulator = Emulator::new(ctx.file_path.clone(), frame_tx, debug_tx, gamepad_state);
     
    emulator.load_cart(&ctx.file_path)
        .expect(&format!("Failed to load the ROM file: {}", &ctx.file_path));

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
        emulator.check_debug();
    }
}

fn start_emulation(mut ctx: MutexGuard<'_, EmuContext>) {
    ctx.start();
}