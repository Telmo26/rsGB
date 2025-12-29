use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::{Arc, Mutex}};

use eframe::egui::{self, Key};
use rs_gb_core::{
    Button,
    settings::{SaveLocation, Settings, SpeedOption},
};

pub const XRES: usize = 160;
pub const YRES: usize = 144;

pub const FRAME_SIZE: usize = XRES as usize * YRES as usize;

pub struct AppSettings {
    inner_settings: Arc<Mutex<InnerSettings>>,
}

impl AppSettings {
    pub fn new() -> AppSettings {
        AppSettings {
            inner_settings: Arc::new(Mutex::new(InnerSettings::new())),
        }
    }

    pub fn emu_settings(&self) -> Settings {
        let settings = self.inner_settings.lock().unwrap();
        settings.emu_settings.clone()
    }

    pub fn key_map(&self) -> HashMap<Key, Button> {
        let lock = self.inner_settings.lock().unwrap();
        lock.key_map.clone()
    }

    pub fn render(&self, ctx: &egui::Context) {
        let mut lock = self.inner_settings.lock().unwrap();

        let mut is_save_next_to_rom = matches!(lock.emu_settings.save_location, SaveLocation::GameLoc);

        egui::SidePanel::left("emulation_settings").show(ctx, |ui| {
            ui.heading("Emulation Settings");

            ui.separator();

            ui.label("Emulation speed");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut lock.emu_settings.speed, SpeedOption::Normal, "1x");
                ui.selectable_value(&mut lock.emu_settings.speed, SpeedOption::X2, "2x");
                ui.selectable_value(&mut lock.emu_settings.speed, SpeedOption::X3, "3x");
                ui.selectable_value(&mut lock.emu_settings.speed, SpeedOption::X4, "4x");
            });

            ui.add_space(20.0);

            ui.label("Save Folder");
            if ui.checkbox(&mut is_save_next_to_rom, "Save next to the rom file").changed() {
                lock.emu_settings.save_location = if is_save_next_to_rom {
                    SaveLocation::GameLoc
                } else {
                    SaveLocation::SaveFolder(PathBuf::new())
                };
            }

            if let SaveLocation::SaveFolder(ref mut path_buf) = lock.emu_settings.save_location {
                ui.horizontal(|ui| {
                    let mut path_str = path_buf.to_string_lossy().to_string();
                    
                    if ui.text_edit_singleline(&mut path_str).changed() {
                        *path_buf = PathBuf::from_str(&path_str).expect("Invalid UTF-8 path typed")
                    }

                    if ui.button("📂").clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            *path_buf = folder;
                        }
                    }
                });
            }
        });

        egui::SidePanel::right("app_settings").show(ctx, |ui| {
            ui.heading("App Settings");

            ui.separator();
        });
    }
}

impl Clone for AppSettings {
    fn clone(&self) -> Self {
        AppSettings {
            inner_settings: self.inner_settings.clone(),
        }
    }
}

struct InnerSettings {
    emu_settings: Settings,
    key_map: HashMap<Key, Button>,
}

impl InnerSettings {
    pub fn new() -> InnerSettings {
        let mut key_map = HashMap::with_capacity(8);

        key_map.insert(Key::Z, Button::A);
        key_map.insert(Key::X, Button::B);
        key_map.insert(Key::P, Button::START);
        key_map.insert(Key::M, Button::SELECT);
        key_map.insert(Key::ArrowUp, Button::UP);
        key_map.insert(Key::ArrowRight, Button::RIGHT);
        key_map.insert(Key::ArrowLeft, Button::LEFT);
        key_map.insert(Key::ArrowDown, Button::DOWN);

        InnerSettings {
            emu_settings: Settings::default(),
            key_map,
        }
    }
}
