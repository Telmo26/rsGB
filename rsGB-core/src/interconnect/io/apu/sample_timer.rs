pub struct SampleTimer {
    accumulator: f64,
    step: f64, // how many samples per APU tick
}

impl SampleTimer {
    pub fn new(apu_clock_hz: f64, sample_rate: f64) -> Self {
        // Each APU tick advances the equivalent of sample_rate / apu_clock samples
        let step = sample_rate / apu_clock_hz;
        Self {
            accumulator: 0.0,
            step,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.accumulator += self.step;
        if self.accumulator >= 1.0 {
            self.accumulator -= 1.0;
            true // emit a sample
        } else {
            false
        }
    }
}
