#[derive(Debug, Default)]
pub struct Timer {
    period: u16,
    value: u16,
}

impl Timer {
    pub fn new(period: u16) -> Timer {
        Timer {
            period,
            ..Timer::default()
        }
    }

    pub fn tick(&mut self) -> bool {
        self.value = self.value.wrapping_sub(1);
        if self.value == 0 {
            self.reset();
            return true
        }
        false
    }

    pub fn set_period(&mut self, period: u16) {
        self.period = period;
    }

    pub fn reset(&mut self) {
        self.value = self.period;
    }
}