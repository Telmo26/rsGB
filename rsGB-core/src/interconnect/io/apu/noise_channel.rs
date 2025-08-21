#[derive(Debug, Default)]
pub struct NoiseChannel {
    length_timer: u8,
    volume_enveloppe: u8,
    freq_randomness: u8,
    control: u8,

    // Internal values
    enabled: bool,
}

impl NoiseChannel {
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF20 => 0xFF,
            0xFF21 => self.volume_enveloppe,
            0xFF22 => self.freq_randomness,
            0xFF23 => self.control | 0xBF,
            _ => unreachable!()
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF20 => self.length_timer = value & 0x3F,
            0xFF21 => self.volume_enveloppe = value,
            0xFF22 => self.freq_randomness = value,
            0xFF23 => self.control = value & 0xC0,
            _ => unreachable!()
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn power_off(&mut self) {
        self.length_timer = 0;
        self.volume_enveloppe = 0;
        self.freq_randomness = 0;
        self.control = 0;
    }
}