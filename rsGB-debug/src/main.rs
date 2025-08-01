use std::{env, ops::Deref, sync::{mpsc::Receiver, Arc, Mutex}, thread};

use rs_gb_core::{run, EmuContext};

use minifb::{Key, Window, WindowOptions, Scale};

mod main_window;
mod debug_window;
use main_window::MainWindow;
use debug_window::DebugWindow;

const COLORS: [u32; 4] = [0x00FFFFFF, 0x00AAAAAA, 0x00555555, 0x00000000];

const CORE_DEBUG: bool = true;

fn main() {
    // PArsing of the arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: rsgb <rom_file>");
        return;
    }

    // Creation of the emulator context
    let context = Arc::new(Mutex::new(EmuContext::new(&args[1], CORE_DEBUG)));
    
    let context1 = Arc::clone(&context);
    let emulator_handle = thread::spawn(move || 
        run(context1)
    );

    // Creation of the windows
    let mut windows = Vec::new();

    let main_window = MainWindow::new();
    windows.push(CustomWindow::MainWindow(main_window));    

    if CORE_DEBUG {
        let debug_rx= context.lock().unwrap().get_debug_rx();
        let debug_window = DebugWindow::new(debug_rx);
        windows.push(CustomWindow::DebugWindow(debug_window));
    }
    
    // Updating the windows
    while windows[0].is_open() && !windows[0].is_key_down(Key::Escape) {
        for w in &mut windows {
            (*w).update();
        }
    }

    // When the window is shut, we stop the emulation
    stop_emulation(context);

    emulator_handle.join().unwrap();
}

fn stop_emulation(context: Arc<Mutex<EmuContext>>) {
    let mut ctx = context.lock().unwrap();
    ctx.stop();
}

enum CustomWindow {
    MainWindow(MainWindow),
    DebugWindow(DebugWindow)
}

impl CustomWindow {
    fn is_open(&self) -> bool {
        if let CustomWindow::MainWindow(w) = self {
            w.is_open()
        } else {
            false
        }
    }

    fn is_key_down(&self, key: Key) -> bool {
        if let CustomWindow::MainWindow(w) = self {
            w.is_key_down(key)
        } else {
            false
        }
    }

    fn update(&mut self) {
        match self {
            CustomWindow::MainWindow(w) => w.update(),
            CustomWindow::DebugWindow(dbg_w) => dbg_w.update()
        }
    }
}