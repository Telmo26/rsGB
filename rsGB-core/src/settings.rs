pub struct Settings {
    speed: SpeedOption,
}

impl Settings {
    pub fn default() -> Settings {
        Settings {
            speed: SpeedOption::NORMAL
        }
    }

    pub fn set_speed(&mut self, speed: SpeedOption) {
        self.speed = speed;
    }

    pub fn get_speed(&self) -> u8 {
        self.speed as u8
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SpeedOption {
    NORMAL = 1,
    X2 = 2,
    X3 = 3,
    X4 = 4,
}