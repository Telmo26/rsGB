use std::{env, sync::{Arc, Mutex}, thread};

use rs_gb::{run, EmuContext};

use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 160;
const HEIGHT: usize = 140;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: rsgb <rom_file>");
        return;
    }
    let context = Arc::new(Mutex::new(EmuContext::new(&args[1])));
    
    let context1 = Arc::clone(&context);
    let emulator_handle = thread::spawn(move || 
        run(context1)
    );

    let buffer = [0x000000; WIDTH* HEIGHT];

    let mut window = Window::new(
        "rsGB - A GameBoy Emulator in Rust",
        WIDTH, 
        HEIGHT, 
        WindowOptions {
            scale: minifb::Scale::X8,
            ..WindowOptions::default()
        }).unwrap();
    
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }

    // When the window is shut, we stop the emulation
    stop_emulation(context);

    emulator_handle.join().unwrap();
}

fn stop_emulation(context: Arc<Mutex<EmuContext>>) {
    let mut ctx = context.lock().unwrap();
    ctx.stop();
}
