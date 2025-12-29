use eframe::egui;
use rs_gb_core::DebugInfo;

pub struct Debugger {
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {  }
    }

    /// Renders the entirety of the debugger window and
    pub fn render(&mut self, ctx: &egui::Context, debug_info: DebugInfo) -> bool {
        let mut stay_open = true;

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.input(|i| i.viewport().close_requested()) {
                // Tell parent to close us.
                stay_open = false;
            }
        });
        stay_open
    }
}