use std::{env, sync::{Arc, Mutex}, thread};

use ringbuf::traits::Consumer;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use minifb::Key;

use rs_gb_core::{init, run, EmuContext};

mod main_window;
mod debug_window;
use main_window::MainWindow;
use debug_window::DebugWindow;

const CORE_DEBUG: bool = false;

fn main() {
    // Parsing of the arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: rsgb <rom_file>");
        return;
    }

    // Creation of the emulator context
    // let context = Arc::new(Mutex::new(EmuContext::new(&args[1], CORE_DEBUG)));
    let (context, mut main_communicator, debug_communicator) = init(CORE_DEBUG);

    // Loading the file into the context
    context.lock().unwrap().load_file(&args[1]);

    // Getting the audio receiver and setting up the audio playback
    let mut audio_receiver = main_communicator.get_audio_receiver();
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device detected");
    let config = device.default_output_config().unwrap();

    println!("{config:?}");

    let stream = device.build_output_stream(
        &config.config(), 
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for sample in data.iter_mut() {
                match audio_receiver.try_pop() {
                    Some(audio) => *sample = audio,
                    None => continue,
                }
            }
        }, 
        move |err| {
            eprintln!("Stream error: {:?}", err);
        }, 
        None
    ).unwrap();

    stream.play().unwrap();

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
}