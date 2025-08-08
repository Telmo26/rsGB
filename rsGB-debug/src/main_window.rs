use std::time::{Duration, Instant};

use minifb::{Key, Scale, Window, WindowOptions};
use rs_gb_core::{MainCommunicator, Button};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const SCALE: Scale = Scale::X4;

pub struct MainWindow {
    window: Window,
    comm: MainCommunicator,
    previous_frame_time: Instant,
    frame_count: u8,
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
        window.set_target_fps(60 + 1);

        MainWindow { 
            window, 
            comm,
            previous_frame_time: Instant::now(),
            frame_count: 0,
        }
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    pub fn update(&mut self) {
        self.comm.update_button(Button::A, self.window.is_key_down(Key::Z));

        self.comm.update_button(Button::B, self.window.is_key_down(Key::X));

        self.comm.update_button(Button::UP, self.window.is_key_down(Key::Up));

        self.comm.update_button(Button::DOWN, self.window.is_key_down(Key::Down));

        self.comm.update_button(Button::LEFT, self.window.is_key_down(Key::Left));

        self.comm.update_button(Button::RIGHT, self.window.is_key_down(Key::Right));

        self.comm.update_button(Button::START, self.window.is_key_down(Key::P));

        self.comm.update_button(Button::SELECT, self.window.is_key_down(Key::M));

        if let Some(new_frame) = self.comm.frame_recv(Duration::from_micros(16600)).as_deref() {
            self.window.update_with_buffer(new_frame, WIDTH, HEIGHT).unwrap();
        } else {
            self.window.update();
        }

        // FPS tracking
        self.frame_count += 1;
        let elapsed = self.previous_frame_time.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let fps = self.frame_count as f64 / elapsed.as_secs_f64();
            println!("{:.2} FPS", fps);
            self.frame_count = 0;
            self.previous_frame_time = Instant::now();
        }
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.window.is_key_down(key)
    }
}