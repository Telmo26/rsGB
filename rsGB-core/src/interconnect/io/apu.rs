use ringbuf::{traits::Producer, HeapProd};

mod pulse_channel;
use pulse_channel::PulseChannel;

mod timer;
use timer::Timer;

type Channel = (u8, u8, u8, u8, u8);

pub struct APU {
    // Communication data
    sender: HeapProd<f32>,
    sampling_timer: Timer,

    // APU internals
    div_apu: u8,

    // CH1 Variables
    ch1: PulseChannel,

    ch2: PulseChannel,

    // CH3 Variables
    _ch3: Channel,

    // CH4 Variables
    _ch4: Channel,

    // Global control registers
    master_vol: u8,
    sound_panning: u8,
    audio_master_ctrl: u8,
}

impl APU {
    pub fn new(sender: HeapProd<f32>) -> APU {
        APU {
            sender,
            sampling_timer: Timer::new(95),

            div_apu: 0,
            
            ch1: PulseChannel::default(),

            ch2: PulseChannel::default(),

            _ch3: (0, 0, 0, 0, 0),

            _ch4: (0, 0, 0, 0, 0),

            master_vol: 0,
            sound_panning: 0,
            audio_master_ctrl: 0,
        }
    }

    pub fn tick(&mut self, div_falling_edge: bool) {
        if div_falling_edge {
            self.div_apu = self.div_apu.wrapping_add(1);
            if self.div_apu % 2 == 0 { // Length counter step
                self.ch1.length_tick();
                self.ch2.length_tick();
            }
            
            if self.div_apu % 4 == 0 { // Sweep step
                self.ch1.sweep_tick();
            }
            
            if self.div_apu % 8 == 0 { // Envelope step  
                self.ch1.enveloppe_tick();
                self.ch2.enveloppe_tick();
            }
        }

        self.ch1.tick();
        self.ch2.tick();

        if self.sampling_timer.tick() && self.audio_enabled() {
            let ch1_output = self.ch1.output();
            let ch2_output = self.ch2.output();
            let ch3_output = 0.0;
            let ch4_output = 0.0;

            let left_vol = (self.master_vol >> 4) & 0x07;
            let right_vol = self.master_vol & 0x07;

            let pan = self.sound_panning;

            let mut left = 0.0;
            let mut right = 0.0;

            // CH1
            if pan & 0b0001 != 0 { right += ch1_output; }
            if pan & 0b0001_0000 != 0 { left += ch1_output; }

            // CH2
            if pan & 0b0010 != 0 { right += ch2_output; }
            if pan & 0b0010_0000 != 0 { left += ch2_output; }

            // CH3
            if pan & 0b0100 != 0 { right += ch3_output; }
            if pan & 0b0100_0000 != 0 { left += ch3_output; }

            // CH4
            if pan & 0b1000 != 0 { right += ch4_output; }
            if pan & 0b1000_0000 != 0 { left += ch4_output; }

            // Apply master volume scaling
            left *= left_vol as f32 / 7.0;
            right *= right_vol as f32 / 7.0;

            // Normalize a little to prevent clipping
            left = left.clamp(-1.0, 1.0);
            right = right.clamp(-1.0, 1.0);

            let _ = self.sender.try_push(left);
            let _ = self.sender.try_push(right);
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF10..0xFF15 => self.ch1.write(address - 0xFF10, value),

            0xFF15..0xFF1A => self.ch2.write(address - 0xFF15, value),

            0xFF24 => self.master_vol = value,
            0xFF25 => self.sound_panning = value,
            0xFF26 => self.audio_master_ctrl = value,
            
            _ => println!("Unimplemented audio register {address:X}")
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF10..0xFF15 => self.ch1.read(address - 0xFF10),

            0xFF15..0xFF1A => self.ch2.read(address - 0xFF15),

            0xFF24 => self.master_vol,
            0xFF25 => self.sound_panning,
            0xFF26 => self.audio_master_ctrl,

            _ => unreachable!()
        }
    }

    fn audio_enabled(&self) -> bool {
        self.audio_master_ctrl & 0b10000000 != 0
    }
}