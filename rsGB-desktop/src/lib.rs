// third party crates imports
use eframe::egui;
use rfd::FileDialog;
use rsgb_core::settings::SpeedOption;

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
            emulation_state: EmulationState::new(cc),
            debugger: Debugger::new(cc),

            app_settings: AppSettings::new(),

            display_debugger: false,
            display_settings: false,
        }
    }
}

impl eframe::App for MyEguiApp {
   fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {        
        egui::Panel::top("Buttons").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        let file = FileDialog::new()
                            .add_filter("GameBoy / GameBoy Color games", &["gb", "gbc"])
                            .pick_file();

                        if let Some(file) = file {
                            if self.emulation_state.cartridge_loaded() {
                                // If another game was already loaded
                                self.emulation_state.reset(frame);
                                self.debugger.reset();
                                self.display_debugger = false;
                            }
                            self.emulation_state.load_cartridge(&file, &self.app_settings);
                        }
                    }
                });

                ui.menu_button("Emulation", |ui| {
                    ui.add_enabled_ui(self.emulation_state.cartridge_loaded(), |ui| {
                        let pause_button = ui.toggle_value(&mut self.emulation_state.paused, "Pause");
                        if pause_button.changed() {
                            self.emulation_state.update_pause_status();
                        } 

                        if ui.button("Stop").clicked() {
                            self.emulation_state.reset(frame);
                            self.debugger.reset();
                            self.display_debugger = false;
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
                ui.show_viewport_immediate(
                    egui::ViewportId::from_hash_of("settings"), 
                    egui::ViewportBuilder::default()
                        .with_always_on_top()                       
                        .with_title("Settings"),
                    |ui, _class| {
                        self.display_settings = self.app_settings.render(ui);
                    }
                );
            }

            if self.display_debugger {
                if !self.debugger.has_game_info() {
                    let game_info = self.emulation_state.get_game_info();
                    self.debugger.load_game_info(game_info);
                }

                ui.show_viewport_immediate(
                    egui::ViewportId::from_hash_of("debugger"), 
                    egui::ViewportBuilder::default()
                        .with_always_on_top()
                        .with_resizable(false)
                        .with_title("Debugger")
                        .with_inner_size((1000.0, 740.0)), 
                    |ui, _class| {
                        let (debug_info, tiles) = self.emulation_state.get_debug_info();
                        self.display_debugger = self.debugger.render(ui, debug_info, tiles);
                    }
                )
            }
        });

        if self.emulation_state.cartridge_loaded() {
            self.emulation_state.render(ui, frame, &self.app_settings);
        }
   }
}