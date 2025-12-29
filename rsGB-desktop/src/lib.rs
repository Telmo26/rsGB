use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

// third party crates imports
use eframe::egui;
use rfd::FileDialog;

// child modules
mod utils;
mod emulation;

use crate::{
    emulation::EmulationState, utils::AppSettings
};


pub struct MyEguiApp {
    emulation_state: Option<EmulationState>,
    app_settings: AppSettings,

    display_settings: Arc<AtomicBool>,
}

impl MyEguiApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        MyEguiApp { 
            emulation_state: None,
            app_settings: AppSettings::new(),

            display_settings: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl eframe::App for MyEguiApp {
   fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {        
        ctx.request_repaint();

        egui::TopBottomPanel::top("Buttons").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        let file = FileDialog::new()
                            .add_filter("GameBoy / GameBoy Color games", &["gb", "gbc"])
                            .set_directory(".")
                            .pick_file();

                        if let Some(file) = file {
                            self.emulation_state = Some(EmulationState::new(ctx, &file));
                        }
                    }
                });

                if ui.button("Settings").clicked() {
                    self.display_settings.store(true, Ordering::Relaxed);
                }
            });
            
            if self.display_settings.load(Ordering::Relaxed) {
                let show_settings = self.display_settings.clone();
                let settings = self.app_settings.clone();
                ctx.show_viewport_deferred(
                    egui::ViewportId::from_hash_of("settings"), 
                    egui::ViewportBuilder::default()
                        .with_always_on_top()
                        .with_title("Settings"),
                    move |ctx, _class| {
                        settings.render(ctx);

                        egui::CentralPanel::default().show(ctx, |ui| {
                            if ui.input(|i| i.viewport().close_requested()) {
                                // Tell parent to close us.
                                show_settings.store(false, Ordering::Relaxed);
                            }
                        });
                    }
                );
            }
        });

        if let Some(emu_state) = &mut self.emulation_state {
            emu_state.render(ctx, &self.app_settings);
        }
   }
}