use std::{env, sync::{mpsc::Receiver, Arc, Mutex}, thread};

use rs_gb_core::{run, EmuContext};

use minifb::{Key, Window, WindowOptions, Scale};

mod debug_window;
use debug_window::DebugWindow;

const WIDTH: usize = 160;
const HEIGHT: usize = 140;

const SCALE: Scale = Scale::X4;

const COLORS: [u32; 4] = [0x00FFFFFF, 0x00AAAAAA, 0x00555555, 0x00000000];

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

    let buffer = [0x00000000; WIDTH* HEIGHT];

    let mut emu_window = Window::new(
        "rsGB - A GameBoy Emulator in Rust",
        WIDTH, 
        HEIGHT, 
        WindowOptions {
            scale: SCALE,
            ..WindowOptions::default()
        }
    ).unwrap();
        
    emu_window.set_target_fps(60);

    let debug_rx = context.lock().unwrap().get_debug_rx();
    
    let mut debug_window = DebugWindow::new(debug_rx);

    while emu_window.is_open() && !emu_window.is_key_down(Key::Escape) {
        emu_window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        debug_window.update();
    }

    // When the window is shut, we stop the emulation
    stop_emulation(context);

    emulator_handle.join().unwrap();
}

fn stop_emulation(context: Arc<Mutex<EmuContext>>) {
    let mut ctx = context.lock().unwrap();
    ctx.stop();
}