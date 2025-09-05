use std::env;

use ringbuf::traits::Consumer;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use rs_gb_core::ThreadedGameboy;

mod main_window;
// mod debug_window;
use main_window::MainWindow;
// use debug_window::DebugWindow;

const CORE_DEBUG: bool = false;

fn main() {
    // Parsing of the arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: rsgb <rom_file>");
        return;
    }

    // Creation of the gameboy
    let mut gameboy = ThreadedGameboy::new(&args[1], CORE_DEBUG);

    // Preparation of the audio stream
    let mut audio_receiver = gameboy.audio_receiver();
    let mut previous_audio = (0.0, 0.0);

    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device detected");
    let config = device.default_output_config().unwrap();

    let stream = device.build_output_stream(
        &config.config(), 
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for sample in data.chunks_mut(2) {
                match audio_receiver.try_pop() {
                    Some((left, right)) => {sample[0] = left ; sample[1] = right ; previous_audio = (left, right)}
                    None => {sample[0] = previous_audio.0 ; sample[1] = previous_audio.1},
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

    let main_window = MainWindow::new(gameboy);
    windows.push(CustomWindow::MainWindow(main_window));    

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
}


enum CustomWindow {
    MainWindow(MainWindow),
}

impl CustomWindow {
    fn is_open(&self) -> bool {
        match self {
            CustomWindow::MainWindow(w) => w.is_open(),
        }
    }

    fn is_main(&self) -> bool {
        match self {
            CustomWindow::MainWindow(_) => true,
        }
    }

    fn update(&mut self) {
        match self {
            CustomWindow::MainWindow(w) => w.update(),
        }
    }
}