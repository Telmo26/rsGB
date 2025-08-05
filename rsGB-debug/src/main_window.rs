use std::time::Duration;

use minifb::{Key, Scale, Window, WindowOptions};
use rs_gb_core::MainCommunicator;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const SCALE: Scale = Scale::X4;

type FrameReceiver = std::sync::mpsc::Receiver<[u32; WIDTH * HEIGHT]>;

pub struct MainWindow {
    window: Window,
    comm: MainCommunicator,
}

impl MainWindow {
    pub fn new(comm: MainCommunicator) -> MainWindow {
        let mut window = Window::new(
            "rsGB - A GameBoy Emulator in Rust",
            WIDTH, 
            HEIGHT, 
            WindowOptions {
                scale: SCALE,
                ..WindowOptions::default()
            }
        ).unwrap();
        window.set_target_fps(60);

        MainWindow { 
            window, 
            comm,
        }
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    pub fn update(&mut self) {
        let recv_result = self.comm.frame_recv(Duration::from_micros(16600));

        if let Some(buffer) = recv_result {
            self.window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        } else {
            self.window.update();
        }
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.window.is_key_down(key)
    }

    pub fn dump(&mut self) {
        let _ = self.comm.frame_recv(Duration::from_micros(16600));
    }
}