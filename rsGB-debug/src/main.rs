use std::{env, sync::{Arc, Mutex, MutexGuard}, thread};

use rs_gb_core::{init, run, EmuContext, MainCommunicator, DebugCommunicator};

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
    // let context = Arc::new(Mutex::new(EmuContext::new(&args[1], CORE_DEBUG)));
    let (context, main_communicator, debug_communicator) = init(CORE_DEBUG);

    // Loading the file into the context
    context.lock().unwrap().load_file(&args[1]);

    // Creation of the windows
    let mut windows = Vec::new();

    let main_window = MainWindow::new(main_communicator);
    windows.push(CustomWindow::MainWindow(main_window));    

    if CORE_DEBUG {
        let debug_window = DebugWindow::new(debug_communicator.unwrap());
        windows.push(CustomWindow::DebugWindow(debug_window));
    }

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