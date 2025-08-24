use super::Timer;

#[derive(Debug, Default)]
pub struct WaveChannel {
    // Registers
    dac_enable: u8,
    initial_length_timer: u8,
    output_level: u8,
    period_low: u8,
    period_high_ctrl: u8,

    wave_pattern_ram: [u8; 16],

    // Internal data
    enabled: bool,

    length_timer: u16,

    period_divider: Timer,
    wave_ram_pointer: u8,

    buffer: u8,
}

impl WaveChannel {
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF1A => self.dac_enable | 0x7F,
            0xFF1B => 0xFF,
            0xFF1C => self.output_level | 0x9F,
            0xFF1D => 0xFF,
            0xFF1E => self.period_high_ctrl | 0xBF,

            0xFF30..0xFF40 => {
                if self.enabled {
                    self.wave_pattern_ram[self.wave_ram_pointer as usize / 2]
                } else {
                    self.wave_pattern_ram[address as usize - 0xFF30]
                }
            } 
            _ => unreachable!()
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF1A => {
                self.dac_enable = value & 0x80;
                if !self.is_dac_enabled() { self.enabled = false }
            },
            0xFF1B => {
                self.initial_length_timer = value;
                self.length_timer = 256 - value as u16;
            },
            0xFF1C => self.output_level = value & 0x60,
            0xFF1D => self.period_low = value,
            0xFF1E => {
                let old_length_enable = self.length_enable();
                self.period_high_ctrl = value & 0xC7; // Only bits 7,6,2-0 are writable
                
                if !old_length_enable && self.length_enable() && self.length_timer == 0 {
                    self.enabled = false;
                }

                if self.trigger() {
                    self.enabled = self.is_dac_enabled();
                    
                    if self.length_timer == 0 {
                        self.length_timer = 256;
                    }

                    self.period_divider.set_period((2048 - self.period()) * 2);

                    self.wave_ram_pointer = 0;
                }
            },

            0xFF30..0xFF40 => {
                if !self.enabled {
                    self.wave_pattern_ram[address as usize - 0xFF30] = value;
                }
            }
            _ => unreachable!()
        }
    }

    pub fn tick(&mut self) {
        if self.period_divider.tick() {
            self.wave_ram_pointer = (self.wave_ram_pointer + 1) % 32;
            
            let byte = self.wave_pattern_ram[(self.wave_ram_pointer / 2) as usize];
            self.buffer = if self.wave_ram_pointer % 2 == 0 {
                byte >> 4
            } else {
                byte & 0x0F
            };
        }
    }

    pub fn length_tick(&mut self) {
        if self.length_enable() {
            self.length_timer = self.length_timer.wrapping_sub(1);
            if self.length_timer == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn output(&self) -> f32 {
        if self.enabled {
            let sample = self.buffer >> self.output_level();
            (7.5 - sample as f32) / 7.5
        } else {
            0.0
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn power_off(&mut self) {
        self.dac_enable = 0;
        self.initial_length_timer = 0;
        self.output_level = 0;
        self.period_low = 0;
        self.period_high_ctrl = 0;
        self.enabled = false;
    }

    fn is_dac_enabled(&self) -> bool {
        self.dac_enable & (1 << 7) != 0
    }

    /// This returns the number of right shifts
    /// to apply to the sample read
    fn output_level(&self) -> u8 {
        match (self.output_level & 0b01100000) >> 5 {
            0b00 => 4, // Mute
            0b01 => 0, // 100%
            0b10 => 1, // 50%
            0b11 => 2, // 25%
            _ => unreachable!()
        }
    }

    fn period(&self) -> u16 {
        self.period_low as u16 | ((self.period_high_ctrl as u16 & 0b111) << 8)
    }

    fn length_enable(&self) -> bool {
        self.period_high_ctrl & 0b01000000 != 0
    }

    fn trigger(&self) -> bool {
        self.period_high_ctrl & 0b10000000 != 0
    }
}