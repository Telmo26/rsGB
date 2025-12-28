use std::{cell::RefCell, env, path::PathBuf, rc::Rc};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use ringbuf::traits::{Consumer, Producer, Split};

use rs_gb_core::Gameboy;

mod main_window;
mod debug_window;

use main_window::MainWindow;
use debug_window::DebugWindow;


fn main() {
    // Parsing of the arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: rsgb <rom_file>");
        return;
    }

    let (mut audio_sender, mut audio_receiver) = ringbuf::StaticRb::<(f32, f32), 8192>::default().split();

    let rom_path = PathBuf::from(&args[1]);

    // Creation of the gameboy
    let  gameboy = Gameboy::new(
        &rom_path, 
        rs_gb_core::ColorMode::ARGB, 
        move |sample| { 
            let _ = audio_sender.try_push(sample);
        }
    );

    // Preparation of the audio stream
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
    let mut windows: Vec<Box<dyn CustomWindow>> = Vec::new();

    let gameboy = Rc::new(RefCell::new(gameboy));

    windows.push(Box::new(MainWindow::new(gameboy.clone())));
    windows.push(Box::new(DebugWindow::new(gameboy)));  

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

trait CustomWindow {
    fn is_open(&self) -> bool;
    fn is_main(&self) -> bool;
    fn update(&mut self);
}