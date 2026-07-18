use std::path::PathBuf;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Settings {
    pub speed: SpeedOption,
    pub save_location: SaveLocation,
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

    pub fn set_save_location(&mut self, save_location: SaveLocation) {
        self.save_location = save_location;
    }

    pub fn get_save_location(&self) -> &SaveLocation {
        &self.save_location
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SpeedOption {
    #[default]
    Normal = 1,
    X2 = 2,
    X3 = 3,
    X4 = 4,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum SaveLocation {
    #[default]
    GameLoc,
    SaveFolder(PathBuf)
}