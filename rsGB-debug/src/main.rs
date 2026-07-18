use std::{cell::RefCell, env, path::PathBuf, rc::Rc};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use ringbuf::{CachingProd, Arc, StaticRb, traits::{Consumer, Producer, Split}};

use rsgb_core::{AudioSink, Gameboy, VideoSink, settings::Settings};

mod main_window;
mod debug_window;

use main_window::MainWindow;
use debug_window::DebugWindow;

const AUDIO_SAMPLES: usize = 4096;

struct DebugAS {
    audio_sender: CachingProd<Arc<StaticRb<(f32, f32), AUDIO_SAMPLES>>>
}

impl AudioSink for DebugAS {
    fn push_sample(&mut self, left: f32, right: f32) {
        while let Err(_) = self.audio_sender.try_push((left, right)) {
            continue;
        }
    }
}

struct DebugVS {
    video_input: triple_buffer::Input<[u32; 0x5A00]>,
}

impl VideoSink for DebugVS {
    fn get_mut(&mut self) -> &mut [u32] {
        self.video_input.input_buffer_mut().as_mut_slice()
    }

    fn present(&mut self) {
        self.video_input.publish();
    }
}

fn main() {
    // Parsing of the arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: rsgb <rom_file>");
        return;
    }

    let (audio_sender, mut audio_receiver) = ringbuf::StaticRb::<(f32, f32), AUDIO_SAMPLES>::default().split();
    let (video_input, video_output) = triple_buffer::triple_buffer(&[0u32; 0x5A00]);

    let rom_path = PathBuf::from(&args[1]);
    let audio_sink = DebugAS { audio_sender };
    let video_sink = DebugVS { video_input };

    // Creation of the gameboy
    let mut gameboy = Gameboy::new( 
        rsgb_core::ColorMode::ARGB, 
        audio_sink,
        video_sink,
    );
    gameboy.load_cartridge(&rom_path, &Settings::default());

    // Preparation of the audio stream
    let mut previous_audio = (0.0, 0.0);

    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device detected");
    let config = device
        .default_output_config()
        .unwrap();

    let stream = device.build_output_stream(
        config.config(), 
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

    windows.push(Box::new(MainWindow::new(
        gameboy.clone(), 
        &rom_path.file_stem().unwrap().to_string_lossy(),
        video_output,    
    )));
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