use std::{path::PathBuf, str::FromStr};

use eframe::egui;
use rs_gb_core::settings::SaveLocation;

use crate::settings::AppSettings;

pub fn save_location_widget(settings: &mut AppSettings, ui: &mut egui::Ui) {
    let mut is_save_next_to_rom = matches!(settings.emu_settings.save_location, SaveLocation::GameLoc);
    
    ui.vertical(|ui| {
        ui.label("Save Folder");
        if ui.checkbox(&mut is_save_next_to_rom, "Save next to the rom file").changed() {
            settings.emu_settings.save_location = if is_save_next_to_rom {
                SaveLocation::GameLoc
            } else {
                SaveLocation::SaveFolder(PathBuf::new())
            };
        }

        if let SaveLocation::SaveFolder(ref mut path_buf) = settings.emu_settings.save_location {
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
}