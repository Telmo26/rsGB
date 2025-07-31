use std::{env, sync::{Arc, Mutex}, thread};

use rs_gb_core::{run, EmuContext};

use minifb::{Key, Window, WindowOptions, Scale};

const WIDTH: usize = 160;
const HEIGHT: usize = 140;
const SCALE: Scale = Scale::X4;
const SCREEN_WIDTH: isize = 1024;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: rsgb <rom_file>");
        return;
    }
    let context = Arc::new(Mutex::new(EmuContext::new(&args[1], true)));
    
    let context1 = Arc::clone(&context);
    let emulator_handle = thread::spawn(move || 
        run(context1)
    );

    let buffer = [0x000000; WIDTH* HEIGHT];

    let debug_buffer:[u32; 32768]  = [u32::MAX; (16 * 8) * (32 * 8)];

    let mut emu_window = Window::new(
        "rsGB - A GameBoy Emulator in Rust",
        WIDTH, 
        HEIGHT, 
        WindowOptions {
            scale: SCALE,
            ..WindowOptions::default()
        }).unwrap();

    let mut debug_window = Window::new(
        "Debug Window",
        16 * 8, 
        32 * 8, 
        WindowOptions {
            scale: SCALE,
            ..WindowOptions::default()
        }).unwrap();
    
    debug_window.set_position(50, 0);

    emu_window.set_target_fps(60);
    debug_window.set_target_fps(60);

    while emu_window.is_open() && !emu_window.is_key_down(Key::Escape) {
        emu_window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        debug_window.update_with_buffer(&debug_buffer, 16 * 8, 32 * 8).unwrap();
    }

    // When the window is shut, we stop the emulation
    stop_emulation(context);

    emulator_handle.join().unwrap();
}

fn stop_emulation(context: Arc<Mutex<EmuContext>>) {
    let mut ctx = context.lock().unwrap();
    ctx.stop();
}
