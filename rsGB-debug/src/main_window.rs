use std::time::Duration;

use minifb::{Key, Scale, Window, WindowOptions};
use rs_gb_core::{MainCommunicator, Button};

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const SCALE: Scale = Scale::X4;

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
        self.comm.update_button(Button::A, self.window.is_key_down(Key::Z));

        self.comm.update_button(Button::B, self.window.is_key_down(Key::X));

        self.comm.update_button(Button::UP, self.window.is_key_down(Key::Up));

        self.comm.update_button(Button::DOWN, self.window.is_key_down(Key::Down));

        self.comm.update_button(Button::LEFT, self.window.is_key_down(Key::Left));

        self.comm.update_button(Button::RIGHT, self.window.is_key_down(Key::Right));

        self.comm.update_button(Button::START, self.window.is_key_down(Key::P));

        self.comm.update_button(Button::SELECT, self.window.is_key_down(Key::M));

        let new_frame = *self.comm.frame_recv();
        self.window.update_with_buffer(&new_frame, WIDTH, HEIGHT).unwrap();
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.window.is_key_down(key)
    }
}