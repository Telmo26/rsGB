use std::{cell::RefCell, rc::Rc};

use minifb::{Scale, Window, WindowOptions};
use rs_gb_core::Gameboy;

use crate::CustomWindow;

// Add a pixel between all tiles
const DEBUG_WIDTH: usize = 16 * 8 + 16 + 1; 
const DEBUG_HEIGHT: usize = 24 * 8 + 24 + 1;
const SCALE: Scale = Scale::X4;

const COLORS: [u32; 4] = [0x00FFFFFF, 0x00AAAAAA, 0x00555555, 0x00000000];

pub struct DebugWindow {
    window: Window,
    buffer: [u32; DEBUG_WIDTH * DEBUG_HEIGHT],
    gameboy: Rc<RefCell<Gameboy>>,
}

impl DebugWindow {
    pub fn new(gameboy: Rc<RefCell<Gameboy>>) -> DebugWindow {
        let mut window = Window::new(
                        "Debug Window",
                        DEBUG_WIDTH, 
                        DEBUG_HEIGHT, 
                        WindowOptions {
                            scale: SCALE,
                            ..WindowOptions::default()
                        }).unwrap();

        window.set_target_fps(60);
        let buffer = [0x00000000; DEBUG_WIDTH * DEBUG_HEIGHT];

        window.update_with_buffer(&buffer, DEBUG_WIDTH, DEBUG_HEIGHT).unwrap();
        
        DebugWindow { 
            window,
            buffer,
            gameboy,
        }
    }
}

impl CustomWindow for DebugWindow {
    fn is_main(&self) -> bool {
        false
    }
    
    fn update(&mut self) {
        let gb = self.gameboy.borrow();
        let debug_info = gb.debug();
        let tiles = debug_info.get_tiles();

        for y in 0..24 {
            for x in 0..16 {
                let tile = &tiles[x + y * 16];
                display_tile(&mut self.buffer, x * 9 + 1, y * 9 + 1, tile);
            }
        }

        self.window.update_with_buffer(&self.buffer, DEBUG_WIDTH, DEBUG_HEIGHT).unwrap();
    }

    fn is_open(&self) -> bool {
        self.window.is_open()
    }
}

fn display_tile(buffer: &mut [u32], start_x: usize, start_y: usize, tile: &[u8; 16]) {
    for tile_y in (0..16).step_by(2) {
        let b1 = tile[tile_y];
        let b2 = tile[tile_y + 1];

        for bit in (0..8).rev() {
            let low = ((b1 & (1 << bit)) != 0) as usize;
            let high = ((b2 & (1 << bit)) != 0) as usize;

            let color = high << 1 | low;

            let x = start_x + (7 - bit);
            let y = start_y + tile_y / 2;
            buffer[x + DEBUG_WIDTH * y] = COLORS[color];
        }
    }
}