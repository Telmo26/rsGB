use std::{env, process::exit};

use eframe::{NativeOptions, egui};

const XRES: usize = 160;
const YRES: usize = 144;

const SCALE: usize = 5;

use rs_gb_desktop::MyEguiApp;

fn main() {    
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([(SCALE * XRES + 16) as f32, (SCALE * YRES + 16) as f32])
            .with_title("Egui Emulator Display"),
        ..Default::default()
    };
    let _ = eframe::run_native("My egui App", 
        options, 
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))));
}