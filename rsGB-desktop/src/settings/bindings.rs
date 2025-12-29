use eframe::egui;
use rs_gb_core::Button;

pub fn bindings_widget(settings: &mut super::AppSettings, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.label("Button binding");

        let buttons = [Button::A, Button::B, Button::DOWN, Button::LEFT, Button::RIGHT, Button::SELECT, Button::START, Button::UP];
                
        egui::Grid::new("bindings")
            .num_columns(2)
            .show(ui, |ui| {
                for button in buttons {
                    ui.label(format!("{:?}", button));

                    let bound_key = settings.key_map.iter()
                        .find(|(_, v)| **v == button)
                        .map(|(&k, _)| k);

                    if settings.awaiting_input == Some(button) {
                        if ui.button("Wait for key...").clicked() {
                            settings.awaiting_input = None; // Cancel if clicked again
                        }

                        ui.input(|i| {
                            if let Some(key) = i.keys_down.iter().next() {
                                // We keep the other settings
                                settings.key_map.retain(|&k, _| k != *key);
                                settings.key_map.retain(|_, &mut v| v != button);

                                // We update the setting we want
                                settings.key_map.insert(*key, button);

                                settings.awaiting_input = None;
                            }
                        });
                    } else {
                        let btn_text = match bound_key {
                            Some(k) => format!("{:?}", k),
                            None => "Unbound".to_string(),
                        };

                        if ui.button(btn_text).clicked() {
                            settings.awaiting_input = Some(button);
                        }
                    }
                    ui.end_row();
                }
            });
            
        if settings.awaiting_input.is_some() {
            ui.label(egui::RichText::new("Press a key to bind it...").color(egui::Color32::YELLOW));
        }
    });
}