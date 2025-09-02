mod cart;
mod cpu;
mod dbg;
mod interconnect;
mod ppu;
mod utils;

use std::{
    sync::{
        Arc, Condvar, Mutex, MutexGuard,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{cart::Cartridge, cpu::CPU, dbg::Debugger, interconnect::Interconnect, ppu::PPU};

use ringbuf::{
    HeapCons, HeapProd,
    traits::{Consumer, Producer, Split},
};

pub type Frame = [u32; 0x5A00];
pub type VRAM = [u8; 0x2000];

const TICKS_PER_SAMPLE: u64 = 95;

#[derive(Debug)]
pub enum Button {
    A,
    B,
    START,
    SELECT,
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

struct Devices {
    bus: Interconnect,
    ppu: PPU,

    audio_callback: fn((f32, f32)),
    framebuffer: Option<*mut [u32]>,

    debugger: Option<Debugger>,
    ticks: u64,
    last_sample_tick: u64,
}

unsafe impl Send for Devices {}

impl Devices {
    fn new(bus: Interconnect, ppu: PPU, audio_callback: fn((f32, f32)), debug: bool) -> Devices {
        Devices {
            bus,
            ppu,
            audio_callback,
            framebuffer: None,
            debugger: if debug { Some(Debugger::new()) } else { None },
            ticks: 0,
            last_sample_tick: 0,
        }
    }

    fn incr_cycle(&mut self, cpu_cycles: u16) {
        for _ in 0..cpu_cycles {
            for _ in 0..4 {
                self.ticks += 1;
                self.bus.tick_t();
                if let Some(ptr) = self.framebuffer {
                    unsafe {
                        let fb = &mut *ptr;
                        self.ppu.tick(&mut self.bus, fb);
                    }
                    
                }

                if self.ticks - self.last_sample_tick >= TICKS_PER_SAMPLE {
                    if let Some(sample) = self.bus.apu_output() {
                        (self.audio_callback)(sample)
                    }
                    self.last_sample_tick = self.ticks;
                }
            }
            self.bus.tick_m();
        }
    }

    fn attach_buffer(&mut self, framebuffer: &mut [u32]) {
        if framebuffer.len() < 0x5A00 {
            panic!("Trying to attach framebuffer that is too small!");
        }
        self.framebuffer = Some(framebuffer as *mut [u32])
    }

    fn detach_buffer(&mut self) {
        self.framebuffer = None;
    }
}

pub struct Gameboy {
    cpu: CPU,
    devices: Devices,

    save_path: String,
}

impl Gameboy {
    pub fn new(rom_path: &str, audio_callback: fn((f32, f32)), debug: bool) -> Gameboy {
        let save_path = rom_path.replace(".gb", ".sav");

        let mut bus = Interconnect::new();
        let ppu = PPU::new();

        let cartridge = Cartridge::load(rom_path).unwrap();
        bus.set_cart(cartridge);
        bus.load_save(&save_path);

        let devices = Devices::new(bus, ppu, audio_callback, debug);
        Gameboy {
            cpu: CPU::new(),
            devices,

            save_path,
        }
    }

    pub fn next_frame(&mut self, framebuffer: &mut [u32]) {
        self.devices.attach_buffer(framebuffer);
        while !self.devices.ppu.is_new_frame() {
            self.cpu.step(&mut self.devices);
        }

        if self.devices.bus.need_save() {
            self.devices.bus.save(&self.save_path);
        }
        self.devices.ppu.get_frame();
        self.devices.detach_buffer();
    }

    pub fn update_button(&mut self, button: Button, value: bool) {
        self.devices.bus.update_button(button, value);
    }

    fn enable_threading(&mut self) {
        //-> (Arc<Mutex<Frame>>, Arc<(Mutex<bool>, Condvar)>) {
        self.devices.ppu.enable_threading();
        // let (framebuffer, frame_available) = self.devices.ppu.enable_threading();
        // (framebuffer, frame_available)
    }
}

pub struct ThreadedGameboy {
    _gameboy_thread: JoinHandle<()>,

    frame_recv: HeapCons<Frame>,
    framebuffer: Arc<Mutex<Frame>>,
    frame_available: Arc<(Mutex<bool>, Condvar)>,

    audio_receiver: Option<Receiver<(f32, f32)>>,
    input_send: HeapProd<(Button, bool)>,
}

impl ThreadedGameboy {
    pub fn new(rom_path: &str, debug: bool) -> ThreadedGameboy {
        // let (mut audio_sender, audio_receiver) = ringbuf::HeapRb::<(f32, f32)>::new(8192).split();
        let (audio_sender, audio_receiver) = mpsc::channel();

        let mut gameboy = Gameboy::new(rom_path, |sample| {}, debug);

        let framebuffer = Arc::new(Mutex::new([0_u32; 0x5A00]));
        let framebuffer_emu = framebuffer.clone();

        let frame_available = Arc::new((Mutex::new(false), Condvar::new()));
        let frame_available_emu = frame_available.clone();

        let (input_send, mut input_recv) = ringbuf::SharedRb::new(10).split();

        let (mut frame_send, frame_recv) = ringbuf::SharedRb::new(2).split();

        // let (framebuffer, frame_available) = gameboy.enable_threading();
        gameboy.enable_threading();

        let _gameboy_thread = thread::spawn(move || {
            loop {
                while let Some((button, value)) = input_recv.try_pop() {
                    gameboy.update_button(button, value);
                }
                let (lock, cvar) = &*frame_available_emu;
                let mut frame_ready = lock.lock().unwrap();

                while *frame_ready {
                    frame_ready = cvar.wait(frame_ready).unwrap();
                }

                let mut frame_lock = framebuffer_emu.lock().unwrap();
                gameboy.next_frame(frame_lock.as_mut_slice());
                
                *frame_ready = true;
                cvar.notify_one();
            }
        });
        ThreadedGameboy {
            _gameboy_thread,

            frame_recv,
            framebuffer,
            frame_available,

            audio_receiver: Some(audio_receiver),
            input_send,
        }
    }

    pub fn recv_frame(&mut self, timeout: Duration) -> Option<MutexGuard<'_, [u32; 0x5A00]>> {
        let (lock, cvar) = &*self.frame_available;

        let mut frame_ready = lock.lock().unwrap();

        while !*frame_ready {
            let (new_lock, result) = cvar.wait_timeout(frame_ready, timeout).unwrap();
            frame_ready = new_lock;

            if result.timed_out() && !*frame_ready {
                return None;
            }
        }

        *frame_ready = false;
        cvar.notify_one();

        return Some(self.framebuffer.lock().unwrap())

        // let frame_lock = self.framebuffer.lock().unwrap();
        // Some(*frame_lock)

        // match self.frame_recv.recv_timeout(timeout) {
        //     Ok(frame) => Some(frame),
        //     Err(_) => None,
        // }

        // self.frame_recv.try_pop()
    }

    pub fn audio_receiver(&mut self) -> Receiver<(f32, f32)> {
        self.audio_receiver.take().unwrap()
    }

    pub fn update_button(&mut self, button: Button, value: bool) {
        let _ = self.input_send.try_push((button, value));
    }
}
