// third party crates imports
use eframe::egui;
use rfd::FileDialog;
use rs_gb_core::settings::SpeedOption;

// child modules
mod settings;
mod emulation;
mod debugger;

use crate::{
    emulation::EmulationState, 
    settings::AppSettings,
    debugger::Debugger,
};


pub struct MyEguiApp {
    emulation_state: EmulationState,
    debugger: Debugger,
    
    app_settings: AppSettings,

    display_debugger: bool,
    display_settings: bool,
}

impl MyEguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        MyEguiApp { 
            emulation_state: EmulationState::new(&cc.egui_ctx),
            debugger: Debugger::new(cc),

            app_settings: AppSettings::new(),

            display_debugger: false,
            display_settings: false,
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
                            .pick_file();

                        if let Some(file) = file {
                            if self.emulation_state.cartridge_loaded() {
                                // If another game was already loaded
                                self.emulation_state = EmulationState::new(ctx);
                            }
                            self.emulation_state.load_cartridge(&file, &self.app_settings);
                        }
                    }
                });

                ui.menu_button("Emulation", |ui| {
                    ui.add_enabled_ui(self.emulation_state.cartridge_loaded(), |ui| {
                        if ui.button("Stop").clicked() {
                            self.emulation_state = EmulationState::new(ctx);
                        }

                        ui.menu_button("Speed", |ui| {
                            ui.selectable_value(&mut self.app_settings.emu_settings.speed, SpeedOption::Normal, "1x");
                            ui.selectable_value(&mut self.app_settings.emu_settings.speed, SpeedOption::X2, "2x");
                            ui.selectable_value(&mut self.app_settings.emu_settings.speed, SpeedOption::X3, "3x");
                            ui.selectable_value(&mut self.app_settings.emu_settings.speed, SpeedOption::X4, "4x");
                        });

                        ui.separator();

                        if ui.button("Debugger").clicked() {
                            self.display_debugger = true;
                        }
                    })
                });

                if ui.button("Settings").clicked() {
                    self.display_settings = true;
                }
            });
            
            if self.display_settings {
                ctx.show_viewport_immediate(
                    egui::ViewportId::from_hash_of("settings"), 
                    egui::ViewportBuilder::default()
                        .with_always_on_top()
                        .with_title("Settings"),
                    |ctx, _class| {
                        self.display_settings = self.app_settings.render(ctx);
                    }
                );
            }

            if self.display_debugger {
                ctx.show_viewport_immediate(
                    egui::ViewportId::from_hash_of("debugger"), 
                    egui::ViewportBuilder::default()
                        .with_always_on_top()
                        .with_title("Debugger")
                        .with_inner_size((1000.0, 750.0)), 
                    |ctx, _class| {
                        let debug_info = self.emulation_state.debug_info();
                        self.display_debugger = self.debugger.render(ctx, debug_info);
                    })
            }
        });

        if self.emulation_state.cartridge_loaded() {
            self.emulation_state.render(ctx, &self.app_settings);
        }
   }
}