use std::time::{Duration, Instant};

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Stream};
use iced::{
    event::{self, Status}, keyboard::{key::Named, Key}, time, widget::{column, image::Handle}, Event, Settings, Subscription
};

use ringbuf::traits::Consumer;
use rs_gb_core;
use rs_gb_core::ThreadedGameboy;

fn main() -> iced::Result {
    iced::application("rsGB - A GameBoy Emulator in Rust", MainWindow::update, MainWindow::view)
        .subscription(MainWindow::subscription)
        .settings(Settings {
            antialiasing: true,
            ..Default::default()
        })
        .run()
}

#[derive(Debug)]
enum Message {
    FrameUpdate,
    KeyUpdated(rs_gb_core::Button, bool),
}

struct MainWindow {
    gameboy: ThreadedGameboy,
    _audio_stream: Stream,

    frame_handle: Option<Handle>,
    instant: Instant,
    counter: u8,
}

impl Default for MainWindow {
    fn default() -> MainWindow {
        let mut gameboy = ThreadedGameboy::new("test_roms/zelda.gb", false);
        
        let mut audio_receiver = gameboy.audio_receiver();
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

        MainWindow { gameboy, _audio_stream, frame_handle: None, instant: Instant::now(), counter: 0 }
    }
}

impl MainWindow {
    fn update(&mut self, message: Message) {
        match message {
            Message::FrameUpdate => {
                if let Some(pixels) = self.gameboy.recv_frame(Duration::from_micros(10_000)) {
                    self.counter += 1;

                    let pixels = convert_framebuffer_u32_to_rgba(&pixels);
                    self.frame_handle = Some(Handle::from_rgba(160, 144, pixels));
                } else {
                    // eprintln!("Received None as a frame");
                }

                let elapsed = self.instant.elapsed();
                if elapsed >= Duration::from_secs(1) {
                    let fps = self.counter as f64 / elapsed.as_secs_f64();
                    println!("{:.2} FPS", fps);
                    self.counter = 0;
                    self.instant = Instant::now();
                }
            }
            Message::KeyUpdated(button, value) => self.gameboy.update_button(button, value),
        }
    }

    fn view(&self) -> iced::Element<Message> {
        if let Some(handle) = self.frame_handle.as_ref() {
            column![
                iced::widget::image(handle)
                    .width(160 * 3)
                    .height(144 * 3)
                    .filter_method(iced::widget::image::FilterMethod::Nearest),
            ]
            .into()
        } else {
            column![

            ].into()
        }
        
    }

    fn subscription(&self) -> Subscription<Message> {
        let subscr_key = event::listen_with(|event, status, _| match (event, status) {
            (
                Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }),
                Status::Ignored,
            ) => {
                if let Some(button) = map_key_to_button(&key) {
                    Some(Message::KeyUpdated(button, true))
                } else {
                    None
                }
            }
            (
                Event::Keyboard(iced::keyboard::Event::KeyReleased { key, .. }),
                Status::Ignored,
            ) => {
                if let Some(button) = map_key_to_button(&key) {
                    Some(Message::KeyUpdated(button, false))
                } else {
                    None
                }
            }
            _ => None,
        });

        Subscription::batch(vec![
            subscr_key,
            time::every(Duration::from_micros(33_200)).map(|_| Message::FrameUpdate),
        ])
    }
}

fn map_key_to_button(key: &Key) -> Option<rs_gb_core::Button> {
    match key {
        // Arrow keys for D-pad
        Key::Named(Named::ArrowUp) => Some(rs_gb_core::Button::UP),
        Key::Named(Named::ArrowDown) => Some(rs_gb_core::Button::DOWN),
        Key::Named(Named::ArrowLeft) => Some(rs_gb_core::Button::LEFT),
        Key::Named(Named::ArrowRight) => Some(rs_gb_core::Button::RIGHT),
        
        // Character keys for action buttons
        Key::Character(c) => match c.as_str() {
            "z" | "Z" => Some(rs_gb_core::Button::A),      // Z for A button
            "x" | "X" => Some(rs_gb_core::Button::B),      // X for B button
            "p" | "P" => Some(rs_gb_core::Button::START),  // P for Start
            "m" | "M" => Some(rs_gb_core::Button::SELECT), // M for Select
            _ => None,
        },
        
        _ => None,
    }
}

fn convert_framebuffer_u32_to_rgba(pixels: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(pixels.len() * 4);
    for &px in pixels {
        let r = ((px >> 16) & 0xFF) as u8;
        let g = ((px >> 8) & 0xFF) as u8;
        let b = (px & 0xFF) as u8;
        let a = ((px >> 24) & 0xFF) as u8;
        out.extend_from_slice(&[r, g, b, a]);
    }
    out
}
