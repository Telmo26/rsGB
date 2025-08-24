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
    sweep: u8,
    length_timer_duty_cycle: u8,
    volume_envelope: u8,
    period_low: u8,
    period_high_ctrl: u8,

    // Internal values
    enabled: bool,

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
            0 => self.sweep | 0x80,
            1 => self.length_timer_duty_cycle | 0x3F,
            2 => self.volume_envelope,
            3 => 0xFF,
            4 => self.period_high_ctrl | 0xBF,
            _ => unreachable!()
        }
    }

    pub fn write(&mut self, address: u16, value: u8, first_period: bool) {
        match address {
            0 => self.sweep = value & 0x7F,
            1 => {
                self.length_timer_duty_cycle = value;
                self.length_timer = 64 - self.initial_length_timer();
            },
            2 => {
                self.volume_envelope = value;
                if !self.is_dac_enabled() {
                    self.enabled = false;
                }
            } 
            3 => self.period_low = value,
            4 => {
                let old_length_enable = self.length_enable();
                
                self.period_high_ctrl = value & 0xC7;
                
                if !old_length_enable && self.length_enable() && self.length_timer != 0 {
                    if first_period {
                        self.length_tick();
                    }
                }
                
                if self.trigger(value) {
                    self.enabled = self.is_dac_enabled();

                    if self.length_timer == 0 {
                        self.length_timer = 64;
                        if first_period {
                            self.length_tick();
                        }
                    }

                    self.timer.set_period((2048 - self.period()) * 4);

                    self.enveloppe_direction = self.enveloppe_direction();
                    self.enveloppe_timer = if self.enveloppe_pace() == 0 {
                        8
                    } else {
                        self.enveloppe_pace()
                    };
                    self.enveloppe_pace = self.enveloppe_pace();

                    self.volume = self.initial_volume();
                    self.waveform_pointer = 0;

                    self.shadow_register = self.period();
                    self.sweep_timer = if self.pace() == 0 { 8 } else { self.pace() };
                    self.sweep_enabled = self.pace() != 0 || self.step() != 0;

                    if self.step() != 0 {
                        self.calculate_sweep_frequency();
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

            let sample = if pattern[self.waveform_pointer as usize] {
                self.volume as f32
            } else {
                0.0
            };

            (7.5 - sample) / 7.5
        } else {
            0.0
        }  
    }

    pub fn length_tick(&mut self) {
        if self.length_enable() {
            self.length_timer = self.length_timer.saturating_sub(1);
            if self.length_timer == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn enveloppe_tick(&mut self) {
        self.enveloppe_timer = self.enveloppe_timer.saturating_sub(1);
        if self.enveloppe_timer == 0 && self.enveloppe_pace != 0 {
            if self.enveloppe_direction {
                if self.volume < 15 {
                    self.volume += 1;
                }
            } else {
                self.volume = self.volume.saturating_sub(1);
            }
            self.enveloppe_timer = self.enveloppe_pace;
        }
    }

    pub fn sweep_tick(&mut self) {
        if !self.sweep_enabled {
            return;
        }

        self.sweep_timer = self.sweep_timer.saturating_sub(1);
        if self.sweep_timer == 0 {
            self.sweep_timer = self.pace();

            if self.pace() > 0 {
                self.calculate_sweep_frequency();
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn power_off(&mut self) {
        self.sweep = 0;
        self.length_timer_duty_cycle = 0;
        self.volume_envelope = 0;
        self.period_low = 0;
        self.period_high_ctrl = 0;
        self.enabled = false;
    }

    fn calculate_sweep_frequency(&mut self) {
        let new_freq = {
            let change = self.shadow_register >> self.step();
            if self.direction() { // Increasing
                self.shadow_register.wrapping_add(change)
            } else { // Decreasing
                self.shadow_register.wrapping_sub(change)
            }
        };
        if new_freq > 0x7FF {
            self.enabled = false;
        } else if self.step() != 0 {
            self.shadow_register = new_freq;
            self.period_low = new_freq as u8;
            self.period_high_ctrl = (self.period_high_ctrl & 0xF8) | ((new_freq >> 8) as u8 & 0x7);

            let second_change = self.shadow_register >> self.step();
            let second_new_freq = if self.direction() {
                self.shadow_register.wrapping_add(second_change)
            } else {
                self.shadow_register.wrapping_sub(second_change)
            };

            if second_new_freq > 0x7FF {
                self.enabled = false;
            }
        }
    }

    fn is_dac_enabled(&self) -> bool {
        // DAC is enabled if the upper 5 bits of NRx2 are non-zero.
        self.volume_envelope & 0xF8 != 0
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

    fn trigger(&self, value: u8) -> bool {
        value & 0b10000000 != 0
    }
}