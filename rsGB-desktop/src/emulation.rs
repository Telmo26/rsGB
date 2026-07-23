use std::{path::PathBuf, sync::{self, Arc, Condvar, Mutex, mpsc::{Receiver, Sender}}, thread::{self, JoinHandle}, time::{Duration, Instant}};

// 3rd party crates
use cpal::{Stream, traits::{DeviceTrait, HostTrait, StreamTrait}};
use eframe::egui;
use ringbuf::{CachingProd, StaticRb, traits::{Consumer, Producer, Split}};

// local crate import
use rsgb_core::{AudioSink, ColorMode, DebugInfo, GameInfo, Gameboy, InputState, VideoSink, settings::Settings};

use crate::settings::{AppSettings, FRAME_SIZE, XRES, YRES};

mod wgpu_state;
use wgpu_state::WgpuState;

const AUDIO_SAMPLES: usize = 1024;

struct DesktopAS {
    audio_input: CachingProd<Arc<StaticRb<(f32, f32), AUDIO_SAMPLES>>>,
    sync: Arc<(Mutex<()>, Condvar)> 
}

impl AudioSink for DesktopAS {
    fn push_sample(&mut self, left: f32, right: f32) {
        while let Err(_) = self.audio_input.try_push((left, right)) {
            let guard = self.sync.0.lock().unwrap();
            let _ = self.sync.1.wait_timeout(guard, Duration::from_millis(5));
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

enum EmulationMessage {
    Pause,
    Continue,
    Reset,
    LoadRom(PathBuf),
    Input(InputState),
    Settings(Settings),
    GetDebugInfo,
    GetGameInfo,
}

#[derive(Debug)]
enum EmulationResponse {
    DebugInfo((DebugInfo, Option<Vec<[u8; 16]>>)),
    GameInfo(GameInfo)
}

pub struct EmulationState {
    _gameboy_thread: JoinHandle<()>,
    last_settings: Settings,
    cartridge_loaded: bool,

    video_output: triple_buffer::Output<[u32; FRAME_SIZE]>,
    wgpu_state: WgpuState,

    message_tx: Sender<EmulationMessage>,
    response_rx: Receiver<EmulationResponse>,

    _audio_stream: Stream,
    
    frame_count: u32,
    previous_frame_time: Instant,

    pub paused: bool,
}

impl EmulationState {
    pub fn new(cc: &eframe::CreationContext<'_>) -> EmulationState {
        // We create the audio and video communication channels
        let (audio_input, mut audio_output) = ringbuf::StaticRb::<(f32, f32), AUDIO_SAMPLES>::default().split();
        let (video_input, video_output) = triple_buffer::triple_buffer(&[0u32; FRAME_SIZE]);

        // We then create the emulation configuration channels
        let (message_tx, message_rx) = sync::mpsc::channel();
        let (response_tx, response_rx) = sync::mpsc::channel();

        let audio_sync = Arc::new((Mutex::new(()), Condvar::new()));

        // These implement the required traits for the gameboy
        let audio_sink = DesktopAS { audio_input, sync: audio_sync.clone() };
        let video_sink = DesktopVS { video_input };

        let mut gameboy = Gameboy::new( 
            ColorMode::ARGB, 
            audio_sink,
            video_sink
        );

        let ctx = cc.egui_ctx.clone();

        let _gameboy_thread = thread::spawn(move || {
            let mut running = false;
            let mut paused = false;
            let mut settings = rsgb_core::settings::Settings::default();
            loop {
                while let Ok(m) = message_rx.try_recv() {
                    match m {
                        EmulationMessage::Continue => paused = false,
                        EmulationMessage::Pause => paused = true,
                        EmulationMessage::Reset => {
                            gameboy.reset();
                            running = false;
                        }
                        EmulationMessage::LoadRom(rom_path) => {
                            gameboy.load_cartridge(&rom_path, &settings);
                            running = true;
                        },
                        EmulationMessage::Input(is) => if running {
                            gameboy.apply_input(is);
                        }
                        EmulationMessage::Settings(s) => settings = s,
                        EmulationMessage::GetDebugInfo => {
                            let dbg = gameboy.debug();
                            let tiles = if dbg.vram_updated() {
                                Some(dbg.get_owned_tiles())
                            } else {
                                None
                            };

                            response_tx
                                .send(EmulationResponse::DebugInfo(
                                    (dbg.get_debug_info(), tiles)
                                ))
                                .expect("Unable to send debug information back to the UI thread")
                        }
                        EmulationMessage::GetGameInfo => {
                            let dbg = gameboy.debug();

                            response_tx
                                .send(EmulationResponse::GameInfo(
                                    dbg.get_game_info()
                                ))
                                .expect("Unable to send debug information back to the UI thread")
                        }
                    }
                }

                if running && !paused {
                    gameboy.next_frame(&settings);
                    ctx.request_repaint_after(Duration::from_micros(16_600));
                }
            }
        });

        // We create the WGPU state for our framebuffer texture
        let wgpu_state = WgpuState::new(cc);

        // Preparation of the audio stream
        let mut previous_audio = (0.0, 0.0);

        let host = cpal::default_host();
        let device = host.default_output_device().expect("No output device detected");
        let supported_range = device
            .supported_output_configs()
            .unwrap()
            .find(|c| {
                c.channels() == 2
                    && c.sample_format() == cpal::SampleFormat::F32
                    && c.min_sample_rate() <= 44_100
                    && c.max_sample_rate() >= 44_100
            })
            .expect("Device does not support stereo float32 44.1kHz output");

        let config = supported_range.with_sample_rate(44_100).config();

        let _audio_stream = device.build_output_stream(
            config, 
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut popped = false;
                for sample in data.chunks_mut(2) {
                    match audio_output.try_pop() {
                        Some((left, right)) => {
                            sample[0] = left; 
                            sample[1] = right; 
                            previous_audio = (left, right);
                            popped = true;
                        }
                        None => {
                            sample[0] = previous_audio.0; 
                            sample[1] = previous_audio.1
                        },
                    }
                }

                if popped {
                    audio_sync.1.notify_one();
                }
            }, 
            move |err| {
                eprintln!("Stream error: {:?}", err);
            }, 
            None
        ).unwrap();

        _audio_stream.play().unwrap();
        EmulationState { 
            _gameboy_thread,
            last_settings: Settings::default(),
            cartridge_loaded: false,

            video_output,
            wgpu_state,

            message_tx,
            response_rx,

            _audio_stream,

            frame_count: 0,
            previous_frame_time: Instant::now(),

            paused: false,
        }
    }

    pub fn load_cartridge(&mut self, rom_path: &PathBuf, app_settings: &AppSettings) {
        self.update_settings(app_settings.emu_settings());
        self.message_tx.send(EmulationMessage::LoadRom(
                rom_path.clone()
            ))
            .expect("Unable to send rom to the emulation thread");
        self.cartridge_loaded = true;
    }

    pub fn cartridge_loaded(&self) -> bool {
        self.cartridge_loaded
    }

    pub fn render(&mut self, ui: &mut egui::Ui, frame: &eframe::Frame, app_settings: &AppSettings) {
        self.update_settings(app_settings.emu_settings());

        let mut input = InputState::default();

        ui.input(|i | {
            for (key, button) in app_settings.key_map() {
                input.update(*button, i.key_down(*key));
            }
        });

        self.message_tx.send(EmulationMessage::Input(input))
            .expect("Unable to send input state to the emulation thread");

        if self.video_output.update() {
            let pixels = self.video_output.read();

            self.wgpu_state.update(frame, pixels);

            self.frame_count += 1;
        }
        
        let elapsed = self.previous_frame_time.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let fps = self.frame_count as f64 / elapsed.as_secs_f64();
            println!("{:.2} FPS", fps);
            self.previous_frame_time = Instant::now();
            self.frame_count = 0;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.centered_and_justified(|ui| {
                let available_width = ui.available_width();
                let x_scale = (available_width / XRES as f32).floor();

                let available_height = ui.available_height();
                let y_scale = (available_height / YRES as f32).floor();

                let scale = x_scale.min(y_scale);

                let size = egui::vec2(XRES as f32 * scale, YRES as f32 * scale);

                ui.add(
                    egui::Image::new((*self.wgpu_state.texture_id(), size))
                        .texture_options(egui::TextureOptions::NEAREST)
                );
            });
        });
    }

    pub fn update_pause_status(&self) {
        let res = if self.paused {
            self.message_tx.send(EmulationMessage::Pause)
        } else {
            self.message_tx.send(EmulationMessage::Continue)
        };
        res.expect("Unable to send pause status to the emulation thread");        
    }

    pub fn reset(&mut self, frame: &eframe::Frame) {
        self.message_tx.send(EmulationMessage::Reset)
            .expect("Unable to send reset signal to the emulation thread");

        self.wgpu_state.update(frame, &[0u32; FRAME_SIZE]);
        self.cartridge_loaded = false;

        self.frame_count = 0;
        self.previous_frame_time = Instant::now();
    }

    pub fn get_game_info(&self) -> GameInfo {
        self.message_tx.send(EmulationMessage::GetGameInfo)
            .expect("Unable to send game info request to emulation thread");

        let response = self.response_rx.recv()
            .expect("Failure while waiting for game info response");

        match response {
            EmulationResponse::GameInfo(data) => data,
            _ => unreachable!()
        }
    }

    pub fn get_debug_info(&self) -> (DebugInfo, Option<Vec<[u8 ; 16]>>) {
        self.message_tx.send(EmulationMessage::GetDebugInfo)
            .expect("Unable to send debug request to emulation thread");

        let response = self.response_rx.recv()
            .expect("Failure while waiting for debug info response");

        match response {
            EmulationResponse::DebugInfo(data) => data,
            _ => unreachable!()
        }
    }

    fn update_settings(&mut self, new_settings: &Settings) {
        if self.last_settings != *new_settings {
            self.last_settings = new_settings.clone();
            self.message_tx.send(EmulationMessage::Settings(
                self.last_settings.clone()
            ))
            .expect("Unable to send updated settings to the emulator thread");
        }
    }
}