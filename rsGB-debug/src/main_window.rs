use minifb::{Key, Scale, Window, WindowOptions};

const WIDTH: usize = 160;
const HEIGHT: usize = 140;
const SCALE: Scale = Scale::X4;

pub struct MainWindow {
    window: Window,
    buffer: [u32; WIDTH* HEIGHT],
}

impl MainWindow {
    pub fn new() -> MainWindow {
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
            buffer: [0x00000000; WIDTH* HEIGHT],
        }
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    pub fn update(&mut self) {
        self.window.update_with_buffer(&self.buffer, WIDTH, HEIGHT).unwrap();
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.window.is_key_down(key)
    }
}