use super::Timer;

#[derive(Debug, Default)]
pub struct NoiseChannel {
    length_timer_reg: u8,
    volume_envelope: u8,
    freq_randomness: u8,
    control: u8,

    // Internal values
    enabled: bool,

    volume: u8,
    enveloppe_pace: u8,
    enveloppe_timer: u8,
    enveloppe_direction: bool,

    length_timer: u8,
    timer: Timer,
}

impl NoiseChannel {
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF20 => 0xFF,
            0xFF21 => self.volume_envelope,
            0xFF22 => self.freq_randomness,
            0xFF23 => self.control | 0xBF,
            _ => unreachable!()
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF20 => {
                self.length_timer_reg = value & 0x3F;
                self.length_timer = 64 - self.initial_length_timer();
            }
            0xFF21 => {
                self.volume_envelope = value;
                if !self.is_dac_enabled() {
                    self.enabled = false;
                }
            } 
            0xFF22 => self.freq_randomness = value,
            0xFF23 => {
                self.control = value & 0xC0;
                if self.trigger(value) {
                    self.enabled = self.is_dac_enabled();

                    if self.length_timer == 0 {
                        self.length_timer = 64;
                    }

                    self.enveloppe_direction = self.enveloppe_direction();
                    self.enveloppe_timer = if self.enveloppe_pace() == 0 {
                        8
                    } else {
                        self.enveloppe_pace()
                    };
                    self.enveloppe_pace = self.enveloppe_pace();

                    self.volume = self.initial_volume();
                }
            }
            _ => unreachable!()
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

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn power_off(&mut self) {
        self.length_timer = 0;
        self.volume_envelope = 0;
        self.freq_randomness = 0;
        self.control = 0;
    }

    fn initial_length_timer(&self) -> u8 {
        self.length_timer_reg & 0b111111
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

    fn is_dac_enabled(&self) -> bool {
        // DAC is enabled if the upper 5 bits of NRx2 are non-zero.
        self.volume_envelope & 0xF8 != 0
    }

    fn length_enable(&self) -> bool {
        self.control & 0b1000000 != 0
    }

    fn trigger(&self, value: u8) -> bool {
        value & 0b10000000 != 0
    }
}