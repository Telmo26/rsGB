use std::{sync::{mpsc::{self, RecvError}, Arc, Mutex}, time::Duration};

pub type Frame = [u32; 0x5A00];
pub type DebugData = [u8; 0x1800];
 
pub type FrameSender = mpsc::SyncSender<Frame>;
pub type FrameReceiver = mpsc::Receiver<Frame>;
 
pub type DebugSender = mpsc::SyncSender<DebugData>;
pub type DebugReceiver = mpsc::Receiver<DebugData>;

pub struct MainCommunicator {
    frame_rx: FrameReceiver,
    gamepad_state: Arc<Mutex<GamepadState>>
}

impl MainCommunicator {
    pub fn frame_recv(&self, timeout: Duration) -> Option<Frame> {
        if timeout == Duration::ZERO {
            Some(self.frame_rx.recv().unwrap())
        } else {
            match self.frame_rx.recv_timeout(timeout) {
                Ok(frame) => Some(frame),
                Err(_) => None,
            }
        }
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
    debug_rx: DebugReceiver,
}

impl DebugCommunicator {
    pub fn vram_recv(&self, timeout: Duration) -> Option<DebugData> {
        if timeout == Duration::ZERO {
            Some(self.debug_rx.recv().unwrap())
        } else {
            match self.debug_rx.recv_timeout(timeout) {
                Ok(debug_data) => Some(debug_data),
                Err(_) => None,
            }
        }
    }
}

pub struct EmuContext {
    pub(crate) file_path: String,

    paused: bool,
    running: bool,

    pub(crate) frame_tx: Option<FrameSender>,
    pub(crate) debug_tx: Option<DebugSender>,
    pub(crate) gamepad_state: Option<Arc<Mutex<GamepadState>>>
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
    let (frame_tx, frame_rx) = mpsc::sync_channel(1);
    let gamepad_state = Arc::new(Mutex::new(GamepadState::new()));

    let (debug_tx, debug_rx);
    if debug {
        let (tx, rx) = mpsc::sync_channel(1);
        debug_tx = Some(tx);
        debug_rx = Some(rx);
    } else {
        debug_tx = None;
        debug_rx = None;
    }

    let gamepad_state1 = Arc::clone(&gamepad_state);
    let main_communicator = MainCommunicator {
        frame_rx,
        gamepad_state: gamepad_state1
    };

    let emu_context = EmuContext {
        file_path: String::new(),

        paused: false,
        running: false,
        frame_tx: Some(frame_tx),
        debug_tx,

        gamepad_state: Some(gamepad_state)
    };

    let mut debug_communicator = None;
    if debug {
        debug_communicator = Some(DebugCommunicator { debug_rx: debug_rx.unwrap() })
    }

    (Arc::new(Mutex::new(emu_context)), main_communicator, debug_communicator)
}