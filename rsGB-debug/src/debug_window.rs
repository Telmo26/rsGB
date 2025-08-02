use std::{sync::mpsc::Receiver, time::Duration};

use minifb::{Scale, Window, WindowOptions};

// Add a pixel between all tiles
const DEBUG_WIDTH: usize = 16 * 8 + 16 + 1; 
const DEBUG_HEIGHT: usize = 24 * 8 + 24 + 1;
const SCALE: Scale = Scale::X4;

const COLORS: [u32; 4] = [0x00FFFFFF, 0x00AAAAAA, 0x00555555, 0x00000000];

pub struct DebugWindow {
    window: Window,
    buffer: [u32; DEBUG_WIDTH * DEBUG_HEIGHT],
    debug_rx: Receiver<[u8; 0x1800]>,
}

impl DebugWindow {
    pub fn new(debug_rx: Receiver<[u8; 0x1800]>) -> DebugWindow {
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
            debug_rx
        }
    }

    pub fn update(&mut self) {
        let recv_result = self.debug_rx.recv_timeout(Duration::from_micros(16600));
        if let Ok(vram) = recv_result {
            let tiles: &[[u8; 16]; 384] = 
                unsafe { &*(vram.as_ptr() as *const [[u8; 16]; 384]) };

            for y in 0..24 {
                for x in 0..16 {
                    let tile = &tiles[x + y * 16];
                    display_tile(&mut self.buffer, x * 9 + 1, y * 9 + 1, tile);
                }
            }

            self.window.update_with_buffer(&self.buffer, DEBUG_WIDTH, DEBUG_HEIGHT).unwrap();
        }

    }

    pub fn dump(&mut self) {
        let _ = self.debug_rx.recv_timeout(Duration::from_micros(16600));
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