use std::time::Duration;

use minifb::{Key, Scale, Window, WindowOptions};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const SCALE: Scale = Scale::X4;

type FrameReceiver = std::sync::mpsc::Receiver<[u32; WIDTH * HEIGHT]>;

pub struct MainWindow {
    window: Window,
    buffer: [u32; WIDTH* HEIGHT],
    frame_rx: FrameReceiver,
}

impl MainWindow {
    pub fn new(frame_rx: FrameReceiver) -> MainWindow {
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
        
        let buffer = [0x00000000; WIDTH* HEIGHT];
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        MainWindow { 
            window, 
            buffer,
            frame_rx,
        }
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    pub fn update(&mut self) {
        match self.frame_rx.recv_timeout(Duration::from_micros(16600)) {
            Ok(b) => self.buffer = b,
            Err(_) => (),
        };
        self.window.update_with_buffer(&self.buffer, WIDTH, HEIGHT).unwrap();
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.window.is_key_down(key)
    }

    pub fn dump(&mut self) {
        let _ = self.frame_rx.recv_timeout(Duration::from_micros(16600));
    }
}