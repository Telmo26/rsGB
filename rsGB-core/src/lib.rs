mod cart;
mod cpu;
mod interconnect;
mod ppu;
mod utils;
mod dbg;

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

// use minifb;

/*
    Emu components :

    |Cart|
    |CPU|
    |Address Bus|
    |PPU|
    |Timer|

*/

type FrameSender = mpsc::SyncSender<[u32; 0x5A00]>;
type FrameReceiver = mpsc::Receiver<[u32; 0x5A00]>;

type DebugSender = mpsc::SyncSender<[u8; 0x1800]>;
type DebugReceiver = mpsc::Receiver<[u8; 0x1800]>;

pub struct EmuContext {
    file_path: String,

    debug: bool,
    paused: bool,
    running: bool,

    frame_tx: Option<FrameSender>,
    frame_rx: Option<FrameReceiver>,

    debug_tx: Option<DebugSender>, 
    debug_rx: Option<DebugReceiver>,
}

impl EmuContext {
    pub fn new(path: &str, debug: bool) -> EmuContext {
        let (debug_tx, debug_rx);
        if debug {
            let (tx, rx) = mpsc::sync_channel(1);
            debug_tx = Some(tx);
            debug_rx = Some(rx);
        } else {
            debug_tx = None;
            debug_rx = None;
        }

        let (frame_tx, frame_rx) = mpsc::sync_channel(1);


        EmuContext {
            file_path: path.to_string(),

            debug,
            paused: false,
            running: false,

            frame_tx: Some(frame_tx),
            frame_rx: Some(frame_rx),

            debug_tx,
            debug_rx
        }
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

    pub fn get_debug_rx(&mut self) -> DebugReceiver {
        let debug_rx = self.debug_rx.take();
        debug_rx.expect("Attempted to get the debug receiver while not in debug mode!")
    }

    pub fn get_frame_rx(&mut self) -> FrameReceiver {
        self.frame_rx.take()
            .expect("Tried to get the frame receiver twice")
    }
}

struct Devices {
    bus: Interconnect,
    ppu: PPU,

    debugger: Option<Debugger>,
    ticks: u64,
}

impl Devices {
    fn new(debug: bool) -> Devices {
        Devices {
            bus: Interconnect::new(),
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

    frame_tx: FrameSender,
    debug_tx: Option<DebugSender>,
}

impl Emulator {
    fn new(frame_tx: FrameSender, debug: bool, debug_tx: Option<DebugSender>) -> Emulator {
        let devices: Devices = Devices::new(debug);

        Emulator {
            cpu: CPU::new(),
            devices,

            frame_tx,
            debug_tx,
        }
    }

    fn load_cart(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cartridge = Cartridge::load(path)?;
        self.devices.bus.set_cart(cartridge);
        Ok(())
    }

    fn step(&mut self) -> bool {
        let res = self.cpu.step(&mut self.devices);

        if let Some(frame) = self.devices.ppu.get_frame() {
            self.frame_tx.send(frame).unwrap();
        }
        res
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
    
    let debug = ctx.debug;
    let debug_tx = ctx.debug_tx.take();

    let frame_tx = ctx.frame_tx.take().unwrap();

    let mut emulator = Emulator::new(frame_tx, debug, debug_tx);
     
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