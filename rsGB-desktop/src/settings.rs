use std::collections::HashMap;

use eframe::egui::{self, Key};

use rs_gb_core::{
    Button,
    settings::Settings,
};

mod bindings;
mod save_location;

use bindings::bindings_widget;
use save_location::save_location_widget;

pub const XRES: usize = 160;
pub const YRES: usize = 144;

pub const FRAME_SIZE: usize = XRES as usize * YRES as usize;

pub struct AppSettings {
    pub(crate) emu_settings: Settings,
    key_map: HashMap<Key, Button>,

    awaiting_input: Option<Button>,
}

impl AppSettings {
    pub fn new() -> AppSettings {
        let mut key_map = HashMap::with_capacity(8);

        key_map.insert(Key::Z, Button::A);
        key_map.insert(Key::X, Button::B);
        key_map.insert(Key::P, Button::START);
        key_map.insert(Key::M, Button::SELECT);
        key_map.insert(Key::ArrowUp, Button::UP);
        key_map.insert(Key::ArrowRight, Button::RIGHT);
        key_map.insert(Key::ArrowLeft, Button::LEFT);
        key_map.insert(Key::ArrowDown, Button::DOWN);

        AppSettings {
            emu_settings: Settings::default(),
            key_map,

            awaiting_input: None,
        }
    }

    pub fn emu_settings(&self) -> &Settings {
        &self.emu_settings
    }

    pub fn key_map(&self) -> &HashMap<Key, Button> {
        &self.key_map
    }

    pub fn render(&mut self, ctx: &egui::Context) -> bool {
        let mut stay_open = true;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("App Settings");

            ui.separator();

            egui::Grid::new("settings_grid")
                .num_columns(3)
                .spacing([40.0, 10.0]) // [horizontal, vertical] spacing
                .min_col_width(200.0)
                .max_col_width(200.0)
                .show(ui, |ui| {
                    bindings_widget(self, ui);

                    save_location_widget(self, ui);

                    ui.end_row();
                    
                });

            if ui.input(|i| i.viewport().close_requested()) {
                // Tell parent to close us.
                    stay_open = false;
            }
        });
        stay_open
    }
}
