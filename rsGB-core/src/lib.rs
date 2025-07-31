mod cart;
mod cpu;
mod interconnect;
mod ppu;
mod utils;
mod dbg;

use std::{
    sync::{Arc, Mutex, MutexGuard, mpsc}, 
    thread, 
    time::Duration
};

use crate::{
    cart::Cartridge, 
    cpu::CPU, 
    ppu::PPU,
    interconnect::{Interconnect, OAMEntry},
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

    debug_tx: Option<mpsc::Sender<[OAMEntry; 40]>>, 
    debug_rx: Option<mpsc::Receiver<[OAMEntry; 40]>>,
}

impl EmuContext {
    pub fn new(path: &str, debug: bool) -> EmuContext {
        let (debug_tx, debug_rx);
        if debug {
            let (tx, rx) = mpsc::channel();
            debug_tx = Some(tx);
            debug_rx = Some(rx);
        } else {
            debug_tx = None;
            debug_rx = None;
        }


        EmuContext {
            file_path: path.to_string(),

            debug,
            paused: false,
            running: false,

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

    pub fn get_debug_rx(&mut self) -> mpsc::Receiver<[OAMEntry; 40]> {
        let debug_rx = self.debug_rx.take();
        debug_rx.expect("Attempted to get the debug receiver while not in debug mode!")
    }
}

struct Devices<'a> {
    bus: Option<&'a mut Interconnect>,
    cpu: Option<&'a mut CPU>,
    ppu: Option<&'a mut PPU>,

    debugger: &'a mut Option<Debugger>,
    ticks: &'a mut u64,
}

impl<'a> Devices<'a> {
    fn incr_cycle(&mut self, cpu_cycles: u16) {
        let n = cpu_cycles * 4;

        if let Some(bus) = self.bus.as_mut() {
            for _ in 0..n {
                *self.ticks += 1;
                bus.timer_tick();
            } 
        }  
    }

    fn bus_read(&self, address: u16) -> u8 {
        if let Some(bus) = self.bus.as_ref() {
            bus.read(address)
        } else {
            panic!("Trying to read through a device that has no bus")
        }
    }

    fn bus_write(&mut self, address: u16, value: u8) {
        if let Some(bus) = self.bus.as_mut() {
            bus.write(address, value)
        } else {
            panic!("Trying to write through a device that has no bus")
        }
    }
}

struct Emulator {
    bus: Interconnect,
    cpu: CPU,
    ppu: PPU,

    debugger: Option<Debugger>,
    ticks: u64,
}

impl Emulator {
    fn new(debug: bool) -> Emulator {
        Emulator {
            bus: Interconnect::new(),
            cpu: CPU::new(),
            ppu: PPU::new(),

            debugger: if debug { Some(Debugger::new()) } else { None },
            ticks: 0,
        }
    }

    fn load_cart(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let cartridge = Cartridge::load(path)?;
        self.bus.set_cart(cartridge);
        Ok(())
    }

    fn step(&mut self) -> bool {
        let (cpu, devices) = self.isolate_cpu();
        
        cpu.step(devices)
    }

    fn isolate_cpu<'a>(&'a mut self) -> (&'a mut CPU, Devices<'a>) {
        unsafe {
            let ptr = self as *mut Emulator;
            let devices = Devices {
                bus: Some(&mut (*ptr).bus),
                cpu: None,
                ppu: Some(&mut (*ptr).ppu),
                debugger: &mut (*ptr).debugger,
                ticks: &mut (*ptr).ticks
            };
            (&mut (*ptr).cpu, devices)
        } 
    }
}

pub fn run(context: Arc<Mutex<EmuContext>>) {
    let ctx: MutexGuard<'_, EmuContext> = context.lock().unwrap();
    
    let mut emulator = Emulator::new(ctx.debug);
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
    }
}

fn start_emulation(mut ctx: MutexGuard<'_, EmuContext>) {
    ctx.start();
}