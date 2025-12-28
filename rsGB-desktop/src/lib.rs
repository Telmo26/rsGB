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
}

impl MyEguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        MyEguiApp { 
            emulation_state: None,
            app_settings: AppSettings::new(),
        }
    }
}

impl eframe::App for MyEguiApp {
   fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {        
        ctx.request_repaint();

        egui::TopBottomPanel::top("Buttons").show(ctx, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    let file = FileDialog::new()
                        .add_filter("GameBoy / GameBoy Color games", &["gb", "gbc"])
                        .set_directory(".")
                        .pick_file();

                    if let Some(file) = file {
                        self.emulation_state = Some(EmulationState::new(ctx, &file))
                    }
                }
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(emu_state) = &mut self.emulation_state {
                emu_state.render(ui);
            }
        });
   }
}