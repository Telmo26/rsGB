use std::time::{Duration, Instant};

use eframe::egui::{self, ColorImage};
use bytemuck::cast_slice;
use cpal::{Stream, traits::{DeviceTrait, HostTrait, StreamTrait}};
use ringbuf::traits::{Consumer, Producer, Split};

use rs_gb_core::{Gameboy, ColorMode};

const XRES: usize = 160;
const YRES: usize = 144;

const FRAME_SIZE: usize = XRES as usize * YRES as usize;

pub struct MyEguiApp {
    gameboy: Gameboy,

    framebuffer: [u32; FRAME_SIZE],
    frame_texture: egui::TextureHandle,

    _audio_stream: Stream,
    
    counter: u32,
    instant: Instant,
}

impl MyEguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>, rom_path: &str) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let (mut audio_sender, mut audio_receiver) = ringbuf::StaticRb::<(f32, f32), 8192>::default().split();

        let  gameboy = Gameboy::new(
            rom_path, 
            ColorMode::ARGB, 
            move |sample| { 
                let _ = audio_sender.try_push(sample);
            },
            false
        );

        let framebuffer = [0; FRAME_SIZE];
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

        MyEguiApp { 
            gameboy, 
            framebuffer, 
            frame_texture,
            _audio_stream,
            counter: 0, 
            instant: Instant::now() 
        }
    }
}

impl eframe::App for MyEguiApp {
   fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.gameboy.next_frame(&mut self.framebuffer);

        let image_size = [XRES, YRES];
        let color_image = ColorImage::from_rgba_unmultiplied(image_size, cast_slice(&self.framebuffer));

        self.frame_texture.set(color_image, egui::TextureOptions::NEAREST);
        
        ctx.request_repaint();

        self.counter += 1;
        let elasped = self.instant.elapsed();
        if elasped >= Duration::from_secs(1) {
            println!("{} FPS", self.counter);
            self.instant = Instant::now();
            self.counter = 0;
        }

       egui::CentralPanel::default().show(ctx, |ui| {
            // Get the screen size to scale the image
            let screen_width = ui.available_width();
            
            // Calculate scale to fit the window while maintaining aspect ratio
            let scale = (screen_width / XRES as f32).floor();

            // Create the Image widget using the texture handle
            let image_widget = egui::Image::new(&self.frame_texture)
                .fit_to_original_size(scale); // Scale to 'display_size'

            // ui.add_space(10.0);
            ui.add(image_widget);
       });
   }
}