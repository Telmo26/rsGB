use super::Timer;

const WAVEFORMS: [[bool; 8]; 4] = [
    [false, false, false, false, false, false, false, true],
    [true, false, false, false, false, false, false, true],
    [true, false, false, false, false, true, true, true],
    [false, true, true, true, true, true, true, false]
];

#[derive(Debug, Default)]
pub struct PulseChannel {
    // Registers
    pub sweep: u8,
    pub length_timer_duty_cycle: u8,
    pub volume_envelope: u8,
    pub period_low: u8,
    pub period_high_ctrl: u8,

    // Internal values
    enabled: bool,
    timer: Timer,
    waveform_pointer: u8,
}

impl PulseChannel {
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0 => self.sweep,
            1 => self.length_timer_duty_cycle,
            2 => self.volume_envelope,
            3 => self.period_low,
            4 => self.period_high_ctrl,
            _ => unreachable!()
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0 => self.sweep = value,
            1 => self.length_timer_duty_cycle = value,
            2 => self.volume_envelope = value,
            3 => self.period_low = value,
            4 => {
                self.period_high_ctrl = value;
                if self.trigger() {
                    self.enabled = true;
                    self.timer.set_period((2048 - self.period()) * 4);
                    self.timer.reset();
                    self.waveform_pointer = 0;
                }
            }
            _ => unreachable!()
        }
    }

    pub fn tick(&mut self) {
        if self.timer.tick() {
            self.waveform_pointer = (self.waveform_pointer + 1) % 8;
        }
    }

    pub fn output(&self) -> f32 {
        let duty = self.wave_duty() as usize;
        let pattern = WAVEFORMS[duty];

        if pattern[self.waveform_pointer as usize] {
            self.initial_volume() as f32
        } else {
            0.0
        }
    }

    fn pace(&self) -> u8 {
        (self.sweep >> 4) & 0b111
    }
    
    /// Returns true if the sweep is increasing,
    /// and false if it is decreasing
    fn direction(&self) -> bool {
        (self.sweep & 0b1000) == 0
    }

    fn step(&self) -> u8 {
        self.sweep & 0b111
    }

    fn wave_duty(&self) -> u8 {
        (self.length_timer_duty_cycle >> 6) & 0b11
    }

    fn initial_length_timer(&self) -> u8 {
        self.length_timer_duty_cycle & 0b111111
    }

    fn initial_volume(&self) -> u8 {
        (self.volume_envelope >> 4) & 0b1111
    }

    /// Returns true if the enveloppe is increasing,
    /// and false if it is decreasing
    fn enveloppe_direction(&self) -> bool {
        self.volume_envelope & 0b1000 != 0
    }

    fn sweep_pace(&self) -> u8 {
        self.volume_envelope & 0b111
    }

    fn period(&self) -> u16 {
        self.period_low as u16 | ((self.period_high_ctrl as u16 & 0b111) << 8)
    }

    fn length_enable(&self) -> bool {
        self.period_high_ctrl & 0b1000000 != 0
    }

    fn trigger(&self) -> bool {
        self.period_high_ctrl & 0b10000000 != 0
    }
}