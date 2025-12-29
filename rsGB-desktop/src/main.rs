use eframe::{NativeOptions, egui};

use rs_gb_desktop::MyEguiApp;

fn main() {    
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Egui Emulator Display")
            .with_inner_size((900.0, 675.0)),
        ..Default::default()
    };
    let _ = eframe::run_native("My egui App", 
        options, 
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))));
}