use std::{sync::{mpsc::{self, RecvError}, Arc, Condvar, Mutex, MutexGuard}, time::Duration};

use crate::{dbg::Debugger, interconnect::Interconnect, ppu::PPU, Devices};

pub type Frame = [u32; 0x5A00];
pub type VRAM = [u8; 0x2000];
 
pub struct MainCommunicator {
    framebuffer: Arc<Mutex<Frame>>,
    frame_sent: Arc<(Mutex<bool>, Condvar)>,
    gamepad_state: Arc<Mutex<GamepadState>>
}

impl MainCommunicator {
    pub fn frame_recv(&self) -> MutexGuard<'_, Frame> {
        let (lock, cvar) = &*self.frame_sent;
        
        let mut frame_ready = lock.lock().unwrap();

        while !*frame_ready {
            frame_ready = cvar.wait(frame_ready).unwrap();
        }

        let frame_lock = self.framebuffer.lock().unwrap();

        *frame_ready = false;
        cvar.notify_one();

        frame_lock
    }

    pub fn update_button(&mut self, button: Button, value: bool) {
        let mut gamepad_lock = self.gamepad_state.lock().unwrap();
        match button {
            Button::A => gamepad_lock.a = value,
            Button::B => gamepad_lock.b = value,
            Button::START => gamepad_lock.start = value,
            Button::SELECT => gamepad_lock.select = value,
            Button::UP => gamepad_lock.up = value,
            Button::DOWN => gamepad_lock.down = value,
            Button::LEFT => gamepad_lock.left = value,
            Button::RIGHT => gamepad_lock.right = value
        }

    }
}

pub struct DebugCommunicator {
    vram: Arc<Mutex<VRAM>>,
}

impl DebugCommunicator {
    pub fn vram_recv(&self) -> MutexGuard<'_, VRAM> {
        let lock = self.vram.lock().unwrap();
        lock
    }
}

pub struct EmuContext {
    pub(crate) file_path: String,

    paused: bool,
    running: bool,

    pub(crate) emulator_devices: Option<Devices>,
}

impl EmuContext {
    pub(crate) fn start(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub(crate) fn is_running(&self) -> bool {
        self.running
    }

    pub(crate) fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn load_file(&mut self, path: &str) {
        self.file_path = path.to_string();
    }
}

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

pub struct GamepadState {
    pub start: bool,
    pub select: bool,
    pub a: bool,
    pub b: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl GamepadState {
    pub fn new() -> GamepadState {
        GamepadState {
            start: false,
            select: false,
            a: false,
            b: false,
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
}

pub fn init(debug: bool) -> (Arc<Mutex<EmuContext>>, MainCommunicator, Option<DebugCommunicator>) {
    let framebuffer = Arc::new(Mutex::new([0; 0x5A00]));
    let frame_sent = Arc::new((Mutex::new(false), Condvar::new()));
    
    let vram = Arc::new(Mutex::new([0; 0x2000]));
    let gamepad_state = Arc::new(Mutex::new(GamepadState::new()));

    let main_communicator = MainCommunicator {
        framebuffer: framebuffer.clone(),
        frame_sent: frame_sent.clone(),
        gamepad_state: gamepad_state.clone(),
    };

    let mut debug_communicator = None;
    if debug {
        debug_communicator = Some(DebugCommunicator {
            vram: vram.clone(),
        })
    }

    let bus = Interconnect::new(vram, gamepad_state);
    let ppu = PPU::new(framebuffer, frame_sent);

    let devices = Devices {
        bus,
        ppu,
        debugger: if debug { Some(Debugger::new()) } else { None },

        ticks: 0,
    };

    let emu_context = EmuContext {
        file_path: String::new(),

        paused: false,
        running: false,
        emulator_devices: Some(devices),
    };

    (Arc::new(Mutex::new(emu_context)), main_communicator, debug_communicator)
}