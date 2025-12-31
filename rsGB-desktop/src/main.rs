#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{NativeOptions, egui};

use rs_gb_desktop::MyEguiApp;

fn main() {    
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("rsGB")
            .with_inner_size((900.0, 675.0)),
        ..Default::default()
    };
    let _ = eframe::run_native("My egui App", 
        options, 
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))));
}