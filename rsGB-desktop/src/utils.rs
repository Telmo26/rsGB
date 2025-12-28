use rs_gb_core::settings::Settings;

pub const XRES: usize = 160;
pub const YRES: usize = 144;
 
pub const FRAME_SIZE: usize = XRES as usize * YRES as usize;

pub struct AppSettings {
    emu_settings: Settings,
}

impl AppSettings {
    pub fn new() -> AppSettings {
        AppSettings { 
            emu_settings: Settings::default(),
        }
    }

    pub fn emu_settings(&self) -> &Settings {
        &self.emu_settings
    }
}