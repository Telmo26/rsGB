use std::{path::PathBuf, time::{Duration, Instant}};

use bytemuck::cast_slice;
// 3rd party crates
use cpal::{Stream, traits::{DeviceTrait, HostTrait, StreamTrait}};
use eframe::egui::{self, ColorImage};
use ringbuf::traits::{Consumer, Producer, Split};

// local crate import
use rs_gb_core::{ColorMode, DebugInfo, Gameboy};

use crate::settings::{AppSettings, FRAME_SIZE, XRES, YRES};

pub struct EmulationState {
    gameboy: Gameboy,

    framebuffer: [u32; FRAME_SIZE],
    frame_texture: egui::TextureHandle,

    _audio_stream: Stream,
    
    counter: u32,
    instant: Instant,
}

impl EmulationState {
    pub fn new(ctx: &egui::Context) -> EmulationState {
        let (mut audio_sender, mut audio_receiver) = ringbuf::StaticRb::<(f32, f32), 8192>::default().split();

        let  gameboy = Gameboy::new( 
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

    pub fn load_cartridge(&mut self, rom_path: &PathBuf, settings: &AppSettings) {
        self.gameboy.load_cartridge(rom_path, &settings.emu_settings);
    }

    pub fn cartridge_loaded(&self) -> bool {
        self.gameboy.cartridge_loaded()
    }

    pub fn render(&mut self, ctx: &egui::Context, settings: &AppSettings) {
        ctx.input(|i | {
            for (key, button) in settings.key_map() {
                self.gameboy.update_button(*button, i.key_down(*key));
            }
        });

        self.gameboy.next_frame(&mut self.framebuffer, &settings.emu_settings());

        let color_image = ColorImage::from_rgba_unmultiplied([XRES, YRES], cast_slice(&self.framebuffer));

        self.frame_texture.set(color_image, egui::TextureOptions::NEAREST);

        self.counter += 1;
        let elasped = self.instant.elapsed();
        if elasped >= Duration::from_secs(1) {
            println!("{} FPS", self.counter);
            self.instant = Instant::now();
            self.counter = 0;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                let available_width = ui.available_width();
                let x_scale = (available_width / XRES as f32).floor();

                let available_height = ui.available_height();
                let y_scale = (available_height / YRES as f32).floor();

                let scale = x_scale.min(y_scale);

                let image_widget = egui::Image::new(&self.frame_texture)
                    .fit_to_original_size(scale);
                ui.add(image_widget);
            });
        });
    }

    pub fn debug_info(&self) -> DebugInfo {
        self.gameboy.debug()
    }
}