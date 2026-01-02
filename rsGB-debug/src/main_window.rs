use std::{cell::RefCell, rc::Rc, time::{Duration, Instant}};

use minifb::{Key, Scale, Window, WindowOptions};
use rs_gb_core::{Button, Gameboy, InputState, settings::Settings};

use crate::CustomWindow;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const SCALE: Scale = Scale::X4;

pub struct MainWindow {
    window: Window,
    gameboy: Rc<RefCell<Gameboy>>,
    settings: Settings,
    framebuffer: [u32; WIDTH * HEIGHT],
    previous_frame_time: Instant,
    frame_count: u8,
}

impl MainWindow {
    pub fn new(gameboy: Rc<RefCell<Gameboy>>, title: &str) -> MainWindow {
        let mut window = Window::new(
            &format!("rsGB - {}", title),
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
            settings: Settings::default(),
            framebuffer: [0; WIDTH * HEIGHT],
            previous_frame_time: Instant::now(),
            frame_count: 0,
        }
    }
}

impl CustomWindow for MainWindow {
    fn is_main(&self) -> bool {
        true
    }

    fn is_open(&self) -> bool {
        self.window.is_open()
    }

    fn update(&mut self) {
        let mut gb = self.gameboy.borrow_mut();

        let mut input = InputState::default();

        input.update(Button::A, self.window.is_key_down(Key::Z));
        input.update(Button::B, self.window.is_key_down(Key::X));
        input.update(Button::UP, self.window.is_key_down(Key::Up));
        input.update(Button::DOWN, self.window.is_key_down(Key::Down));
        input.update(Button::LEFT, self.window.is_key_down(Key::Left));
        input.update(Button::RIGHT, self.window.is_key_down(Key::Right));
        input.update(Button::START, self.window.is_key_down(Key::P));
        input.update(Button::SELECT, self.window.is_key_down(Key::M));

        gb.apply_input(input);
        gb.next_frame(&mut self.framebuffer, &self.settings);

        self.window.update_with_buffer(&self.framebuffer, WIDTH, HEIGHT).unwrap();
        self.frame_count += 1;

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