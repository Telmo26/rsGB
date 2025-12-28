use std::{path::PathBuf, time::{Duration, Instant}};

use bytemuck::cast_slice;
// 3rd party crates
use cpal::{Stream, traits::{DeviceTrait, HostTrait, StreamTrait}};
use eframe::egui::{self, ColorImage};
use ringbuf::traits::{Consumer, Producer, Split};

// local crate import
use rs_gb_core::{Gameboy, ColorMode};

use crate::utils::{AppSettings, FRAME_SIZE, XRES, YRES};

pub struct EmulationState {
    gameboy: Gameboy,

    framebuffer: [u32; FRAME_SIZE],
    frame_texture: egui::TextureHandle,

    _audio_stream: Stream,
    
    counter: u32,
    instant: Instant,
}

impl EmulationState {
    pub fn new(ctx: &egui::Context, rom_path: &PathBuf) -> EmulationState {
        let (mut audio_sender, mut audio_receiver) = ringbuf::StaticRb::<(f32, f32), 8192>::default().split();

        let  gameboy = Gameboy::new(
            rom_path, 
            ColorMode::ARGB, 
            move |sample| { 
                let _ = audio_sender.try_push(sample);
            }
        );

        let framebuffer = [0; FRAME_SIZE];
        let initial_image = ColorImage::new([XRES, YRES], vec![egui::Color32::BLACK; FRAME_SIZE]);

        let frame_texture = ctx.load_texture(
            "emulator_frame", 
            initial_image, 
            egui::TextureOptions::NEAREST,
        );

        // Preparation of the audio stream
        let mut previous_audio = (0.0, 0.0);

        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device detected");
        let config = device.default_output_config().unwrap();

        let _audio_stream = device.build_output_stream(
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

        _audio_stream.play().unwrap();
        EmulationState { 
            gameboy,

            framebuffer,
            frame_texture,

            _audio_stream,

            counter: 0,
            instant: Instant::now(),
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, settings: &AppSettings) {
        self.gameboy.next_frame(&mut self.framebuffer, settings.emu_settings());

        let image_size = [XRES, YRES];
        let color_image = ColorImage::from_rgba_unmultiplied(image_size, cast_slice(&self.framebuffer));

        self.frame_texture.set(color_image, egui::TextureOptions::NEAREST);

        self.counter += 1;
        let elasped = self.instant.elapsed();
        if elasped >= Duration::from_secs(1) {
            println!("{} FPS", self.counter);
            self.instant = Instant::now();
            self.counter = 0;
        }

        // Get the screen size to scale the image
        let screen_width = ui.available_width();
        
        // Calculate scale to fit the window while maintaining aspect ratio
        let scale = (screen_width / XRES as f32).floor();

        // Create the Image widget using the texture handle
        let image_widget = egui::Image::new(&self.frame_texture)
            .fit_to_original_size(scale); // Scale to 'display_size'

        // ui.add_space(10.0);
        ui.add(image_widget);
    }
}