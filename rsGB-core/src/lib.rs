mod cart;
mod cpu;
mod interconnect;
mod ppu;
mod utils;
mod dbg;

use std::{sync::{mpsc::{self, Receiver, Sender}, Arc, Condvar, Mutex, MutexGuard}, thread::{self, JoinHandle}, time::Duration};

use crate::{
    cart::Cartridge, 
    cpu::CPU, 
    ppu::PPU,
    interconnect::Interconnect,
    dbg::Debugger,
};

use ringbuf::{traits::{Consumer, Producer, Split}, HeapCons, HeapProd};

pub type Frame = [u32; 0x5A00];
pub type VRAM = [u8; 0x2000];

#[derive(Debug)]
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
        bus.load_save(&save_path);

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

        let frame = &*self.devices.ppu.get_frame().unwrap();
        frame
    }

    pub fn update_button(&mut self, button: Button, value: bool) {
        self.devices.bus.update_button(button, value);        
    }

    pub fn audio_receiver(&mut self) -> HeapCons<(f32, f32)> {
        self.audio_receiver.take().unwrap()
    }

    fn enable_threading(&mut self) { //-> (Arc<Mutex<Frame>>, Arc<(Mutex<bool>, Condvar)>) {
        self.devices.ppu.enable_threading();
        // let (framebuffer, frame_available) = self.devices.ppu.enable_threading();
        // (framebuffer, frame_available)
    }
}

pub struct ThreadedGameboy {
    _gameboy_thread: JoinHandle<()>,

    frame_recv: Receiver<Frame>,
    // framebuffer: Arc<Mutex<Frame>>,
    // frame_available: Arc<(Mutex<bool>, Condvar)>,

    audio_receiver: Option<HeapCons<(f32, f32)>>,
    input_send: HeapProd<(Button, bool)>
}

impl ThreadedGameboy {
    pub fn new(rom_path: &str, debug: bool) -> ThreadedGameboy {
        let mut gameboy = Gameboy::new(rom_path, debug);

        let audio_receiver = gameboy.audio_receiver();

        let (input_send, mut input_recv) = ringbuf::SharedRb::new(10).split();

        let (frame_send, frame_recv) = mpsc::sync_channel(3);

        // let (framebuffer, frame_available) = gameboy.enable_threading();
        gameboy.enable_threading();

        let _gameboy_thread = thread::spawn(move ||
            loop {
                while let Some((button, value)) = input_recv.try_pop() {
                    gameboy.update_button(button, value);
                }
                let frame = gameboy.next_frame();
                if let Err(_) = frame_send.send(*frame) {
                    break
                }
            }
        );
        ThreadedGameboy { _gameboy_thread, frame_recv, audio_receiver: Some(audio_receiver), input_send }
    }

    pub fn recv_frame(&self, timeout: Duration) -> Option<Frame> {
        // let (lock, cvar) = &*self.frame_available;

        // let mut frame_ready = lock.lock().unwrap();

        // while !*frame_ready {
        //     let (new_lock, result) = cvar.wait_timeout(frame_ready, timeout).unwrap();
        //     frame_ready = new_lock;

        //     if result.timed_out() && !*frame_ready {
        //         return None;
        //     }
        // }

        // *frame_ready = false;
        // cvar.notify_one();

        // let frame_lock = self.framebuffer.lock().unwrap();
        // Some(*frame_lock)
        match self.frame_recv.recv_timeout(timeout) {
            Ok(frame) => Some(frame),
            Err(_) => None,
        }
    }

    pub fn audio_receiver(&mut self) -> HeapCons<(f32, f32)> {
        self.audio_receiver.take().unwrap()
    }

    pub fn update_button(&mut self, button: Button, value: bool) {
        let _ = self.input_send.try_push((button, value));
    }
}