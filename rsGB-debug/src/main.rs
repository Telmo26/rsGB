use std::{env, sync::{Arc, Mutex, MutexGuard}, thread};

use rs_gb_core::{run, EmuContext};

use minifb::Key;

mod main_window;
mod debug_window;
use main_window::MainWindow;
use debug_window::DebugWindow;

const CORE_DEBUG: bool = true;

fn main() {
    // Parsing of the arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: rsgb <rom_file>");
        return;
    }

    // Creation of the emulator context
    let context = Arc::new(Mutex::new(EmuContext::new(&args[1], CORE_DEBUG)));

    // Creation of the windows
    let mut windows = Vec::new();

    let context_lock = context.lock().unwrap();
    generate_windows(&mut windows, context_lock);

    // Launching the emulator
    let context1 = Arc::clone(&context);
    let emulator_handle = thread::spawn(move || 
        run(context1)
    );
    
    // Updating the windows
    while windows.iter().any(|w| w.is_main() && w.is_open()) {
        windows.retain_mut(|window|
            if window.is_open() {
                window.update();
                true
            } else {
                false
            }
        );
    }
    
    // When the window is shut, we stop the emulation and dump the frames
    stop_emulation(context);

    emulator_handle.join().unwrap();
}

fn stop_emulation(context: Arc<Mutex<EmuContext>>) {
    let mut ctx = context.lock().unwrap();
    ctx.stop();
}

fn generate_windows(windows: &mut Vec<CustomWindow>, mut context_lock: MutexGuard<'_, EmuContext>) {
    let frame_rx = context_lock.get_frame_rx();
    let main_window = MainWindow::new(frame_rx);
    windows.push(CustomWindow::MainWindow(main_window));    

    if CORE_DEBUG {
        let debug_rx= context_lock.get_debug_rx();
        let debug_window = DebugWindow::new(debug_rx);
        windows.push(CustomWindow::DebugWindow(debug_window));
    }
}

enum CustomWindow {
    MainWindow(MainWindow),
    DebugWindow(DebugWindow)
}

impl CustomWindow {
    fn is_open(&self) -> bool {
        match self {
            CustomWindow::MainWindow(w) => w.is_open(),
            CustomWindow::DebugWindow(w) => w.is_open(),
        }
    }

    fn is_main(&self) -> bool {
        match self {
            CustomWindow::MainWindow(_) => true,
            _ => false
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

    fn dump(&mut self) {
        match self {
            CustomWindow::MainWindow(w) => w.dump(),
            CustomWindow::DebugWindow(dbg_w) => dbg_w.dump()
        }
    }
}