use std::time::{Duration, Instant};

use minifb::{Key, Scale, Window, WindowOptions};
use rs_gb_core::{Button, ThreadedGameboy};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const SCALE: Scale = Scale::X4;

pub struct MainWindow {
    window: Window,
    gameboy: ThreadedGameboy,
    previous_frame_time: Instant,
    frame_count: u8,
}

impl MainWindow {
    pub fn new(gameboy: ThreadedGameboy) -> MainWindow {
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
            gameboy,
            previous_frame_time: Instant::now(),
            frame_count: 0,
        }
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    pub fn update(&mut self) {
        self.gameboy.update_button(Button::A, self.window.is_key_down(Key::Z));

        self.gameboy.update_button(Button::B, self.window.is_key_down(Key::X));

        self.gameboy.update_button(Button::UP, self.window.is_key_down(Key::Up));

        self.gameboy.update_button(Button::DOWN, self.window.is_key_down(Key::Down));

        self.gameboy.update_button(Button::LEFT, self.window.is_key_down(Key::Left));

        self.gameboy.update_button(Button::RIGHT, self.window.is_key_down(Key::Right));

        self.gameboy.update_button(Button::START, self.window.is_key_down(Key::P));

        self.gameboy.update_button(Button::SELECT, self.window.is_key_down(Key::M));

        if let Some(new_frame) = self.gameboy.recv_frame(Duration::from_micros(16_600)) {
            self.window.update_with_buffer(new_frame.as_slice(), WIDTH, HEIGHT).unwrap();
            self.frame_count += 1;
        } else {
            self.window.update();
        }

        // FPS tracking
        let elapsed = self.previous_frame_time.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let fps = self.frame_count as f64 / elapsed.as_secs_f64();
            println!("{:.2} FPS", fps);
            self.frame_count = 0;
            self.previous_frame_time = Instant::now();
        }
    }
}