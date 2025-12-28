pub const XRES: usize = 160;
pub const YRES: usize = 144;
 
pub const FRAME_SIZE: usize = XRES as usize * YRES as usize;

pub struct AppSettings {}

impl AppSettings {
    pub fn new() -> AppSettings {
        AppSettings {  }
    }
}