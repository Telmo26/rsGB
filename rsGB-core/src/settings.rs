use std::path::PathBuf;

pub struct Settings {
    speed: SpeedOption,
    save_location: SaveLocation,
}

impl Settings {
    pub fn default() -> Settings {
        Settings {
            speed: SpeedOption::Normal,
            save_location: SaveLocation::GameLoc,
        }
    }

    pub fn new(save_folder: PathBuf) -> Settings {
        Settings { 
            speed: SpeedOption::Normal, 
            save_location: SaveLocation::SaveFolder(save_folder), 
        }
    }

    pub fn set_speed(&mut self, speed: SpeedOption) {
        self.speed = speed;
    }

    pub fn get_speed(&self) -> u8 {
        self.speed as u8
    }

    pub fn set_save_location(&mut self, save_location: SaveLocation) {
        self.save_location = save_location;
    }

    pub fn get_save_location(&self) -> &SaveLocation {
        &self.save_location
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpeedOption {
    Normal = 1,
    X2 = 2,
    X3 = 3,
    X4 = 4,
}

#[derive(Debug, Clone)]
pub enum SaveLocation {
    GameLoc,
    SaveFolder(PathBuf)
}