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
    pub enabled: bool,

    sweep_enabled: bool,
    sweep_timer: u8,
    shadow_register: u16,

    volume: u8,
    enveloppe_pace: u8,
    enveloppe_timer: u8,
    enveloppe_direction: bool,

    length_timer: u8,
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

                    if self.length_timer == 0 {
                        self.length_timer = 64 - self.initial_length_timer();
                    }

                    self.timer.set_period((2048 - self.period()) * 4);
                    self.timer.reset();

                    self.enveloppe_direction = self.enveloppe_direction();
                    self.enveloppe_timer = self.enveloppe_pace();
                    self.enveloppe_pace = self.enveloppe_pace();

                    self.volume = self.initial_volume();
                    self.waveform_pointer = 0;

                    self.shadow_register = self.period();
                    self.sweep_timer = self.pace();
                    self.sweep_enabled = self.pace() != 0 || self.step() != 0;

                    if self.step() != 0 {
                        let change = if self.direction() { (self.shadow_register >> self.step()) as i16}
                            else { - ((self.shadow_register >> self.step()) as i16) };
                        let freq = self.shadow_register.wrapping_add_signed(change);
                        if freq > 0x7FF {
                            self.enabled = false;
                        } else {
                            self.shadow_register = freq;
                            self.period_low = freq as u8;
                            self.period_high_ctrl = (self.period_high_ctrl & 0xF8) | ((freq >> 8) as u8 & 0x7);
                        }
                    }
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
        if self.enabled {
            let duty = self.wave_duty() as usize;
            let pattern = WAVEFORMS[duty];

            let sample = self.volume as f32 / 15.0;
            let bipolar = 2.0 * sample - 1.0; 

            if pattern[self.waveform_pointer as usize] {
                bipolar
            } else {
                -bipolar
            }
        } else {
            0.0
        }  
    }

    pub fn length_tick(&mut self) {
        if self.enabled && self.length_enable() {
            self.length_timer -= 1;
            if self.length_timer == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn enveloppe_tick(&mut self) {
        if self.enveloppe_timer == 0 {
            if self.enveloppe_pace != 0 {
                if self.enveloppe_direction {
                    if self.volume < 15 {
                        self.volume += 1;
                    }
                } else {
                    self.volume = self.volume.saturating_sub(1);
                }
                self.enveloppe_timer = self.enveloppe_pace;
            }
        } else {
            self.enveloppe_timer -= 1;
        }
    }

    pub fn sweep_tick(&mut self) {
        if !self.sweep_enabled || self.step() == 0 || self.pace() == 0 {
            return;
        }

        if self.sweep_timer == 0 {
            let change = if self.direction() { (self.shadow_register >> self.step()) as i16}
                                else { - ((self.shadow_register >> self.step()) as i16) };
            let freq = self.shadow_register.wrapping_add_signed(change);
            if freq > 0x7FF {
                self.enabled = false;
            } else if self.step() != 0 {
                self.shadow_register = freq;
                self.period_low = freq as u8;
                self.period_high_ctrl = (self.period_high_ctrl & 0xF8) | ((freq >> 8) as u8 & 0x7);

                let change = if self.direction() { (self.shadow_register >> self.step()) as i16}
                else { - ((self.shadow_register >> self.step()) as i16) };
                let freq = self.shadow_register.wrapping_add_signed(change);
                if freq > 0x7FF {
                    self.enabled = false;
                }
            }
            self.sweep_timer = self.pace();
        } else {
            self.sweep_timer -= 1;
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

    fn enveloppe_pace(&self) -> u8 {
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