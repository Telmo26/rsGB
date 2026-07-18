use std::{path::PathBuf, sync::Arc, time::{Duration, Instant}};

use bytemuck::cast_slice;
// 3rd party crates
use cpal::{Stream, traits::{DeviceTrait, HostTrait, StreamTrait}};
use eframe::egui::{self, ColorImage};
use ringbuf::{CachingProd, StaticRb, traits::{Consumer, Producer, Split}};

// local crate import
use rsgb_core::{AudioSink, ColorMode, DebugInfo, Gameboy, InputState, VideoSink};

use crate::settings::{AppSettings, FRAME_SIZE, XRES, YRES};

const AUDIO_SAMPLES: usize = 4096;

struct DesktopAS {
    audio_sender: CachingProd<Arc<StaticRb<(f32, f32), AUDIO_SAMPLES>>>
}

impl AudioSink for DesktopAS {
    fn push_sample(&mut self, left: f32, right: f32) {
        while let Err(_) = self.audio_sender.try_push((left, right)) {
            continue;
        }
    }
}

struct DesktopVS {
    video_input: triple_buffer::Input<[u32; FRAME_SIZE]>,
}

impl VideoSink for DesktopVS {
    fn get_mut(&mut self) -> &mut [u32] {
        self.video_input.input_buffer_mut().as_mut_slice()
    }

    fn present(&mut self) {
        self.video_input.publish();
    }
}

pub struct EmulationState {
    gameboy: Gameboy<DesktopAS, DesktopVS>,

    video_output: triple_buffer::Output<[u32; FRAME_SIZE]>,
    frame_texture: egui::TextureHandle,

    _audio_stream: Stream,
    
    counter: u32,
    instant: Instant,
}

impl EmulationState {
    pub fn new(cc: &eframe::CreationContext<'_>) -> EmulationState {
        let (audio_sender, mut audio_receiver) = ringbuf::StaticRb::<(f32, f32), AUDIO_SAMPLES>::default().split();
        let (video_input, video_output) = triple_buffer::triple_buffer(&[0u32; FRAME_SIZE]);

        let audio_sink = DesktopAS { audio_sender };
        let video_sink = DesktopVS { video_input };

        let gameboy = Gameboy::new( 
            ColorMode::ARGB, 
            audio_sink,
            video_sink
        );

        let initial_image = ColorImage::new([XRES, YRES], vec![egui::Color32::BLACK; FRAME_SIZE]);

        let frame_texture = cc.egui_ctx.load_texture(
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

        _audio_stream.play().unwrap();
        EmulationState { 
            gameboy,

            video_output,
            frame_texture,

            _audio_stream,

            counter: 0,
            instant: Instant::now(),
        }
    }

    pub fn load_cartridge(&mut self, rom_path: &PathBuf, settings: &AppSettings) {
        self.gameboy.load_cartridge(rom_path, settings.emu_settings());
    }

    pub fn cartridge_loaded(&self) -> bool {
        self.gameboy.cartridge_loaded()
    }

    pub fn render(&mut self, ui: &mut egui::Ui, settings: &AppSettings) {
        let mut input = InputState::default();

        ui.input(|i | {
            for (key, button) in settings.key_map() {
                input.update(*button, i.key_down(*key));
            }
        });

        self.gameboy.apply_input(input);
        self.gameboy.next_frame(settings.emu_settings());

        if self.video_output.update() {
            let color_image = ColorImage::from_rgba_unmultiplied([XRES, YRES], cast_slice(self.video_output.read()));

            self.frame_texture.set(color_image, egui::TextureOptions::NEAREST);

            self.counter += 1;
        }
        
        let elasped = self.instant.elapsed();
        if elasped >= Duration::from_secs(1) {
            println!("{} FPS", self.counter);
            self.instant = Instant::now();
            self.counter = 0;
        }

        egui::CentralPanel::default().show(ui, |ui| {
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

    pub fn debug_info<'a>(&'a self) -> DebugInfo<'a> {
        self.gameboy.debug()
    }

    pub fn reset(&mut self) {
        self.gameboy.reset();
        let color_image = ColorImage::new([XRES, YRES], vec![egui::Color32::BLACK; FRAME_SIZE]);
        self.frame_texture.set(color_image, egui::TextureOptions::NEAREST);

        self.counter = 0;
        self.instant = Instant::now();
    }
}