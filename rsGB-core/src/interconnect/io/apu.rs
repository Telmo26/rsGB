use core::panic;

use ringbuf::{HeapProd, traits::Producer};

mod pulse_channel;
use pulse_channel::PulseChannel;

mod wave_channel;
use wave_channel::WaveChannel;

mod noise_channel;
use noise_channel::NoiseChannel;

mod timer;
use timer::Timer;

mod sample_timer;
use sample_timer::SampleTimer;

pub struct APU {
    // Communication data
    sender: HeapProd<(f32, f32)>,
    sample_timer: SampleTimer,
    output_buffer: Vec<(f32, f32)>,

    // APU internals
    div_apu: u8,

    // CH1 Variables
    ch1: PulseChannel,

    ch2: PulseChannel,

    // CH3 Variables
    ch3: WaveChannel,

    // CH4 Variables
    ch4: NoiseChannel,

    // Global control registers
    master_vol: u8,
    sound_panning: u8,
    audio_master_ctrl: u8,    
}

impl APU {
    pub fn new(sender: HeapProd<(f32, f32)>) -> APU {
        APU {
            sender,
            sample_timer: SampleTimer::new(4_194_304.0, 44_100.0),
            output_buffer: Vec::with_capacity(100),

            div_apu: 0,
            
            ch1: PulseChannel::default(),

            ch2: PulseChannel::default(),

            ch3: WaveChannel::default(),

            ch4: NoiseChannel::default(),

            master_vol: 0,
            sound_panning: 0,
            audio_master_ctrl: 0,
        }
    }

    pub fn tick(&mut self, div_falling_edge: bool) {
        if div_falling_edge {
            self.div_apu = (self.div_apu + 1) % 8;
            if self.div_apu & 1 == 0 { // Length counter step
                self.ch1.length_tick();
                self.ch2.length_tick();
                self.ch3.length_tick();
                self.ch4.length_tick();
            }
            
            if self.div_apu == 2 || self.div_apu == 6 { // Sweep step
                self.ch1.sweep_tick();
            }
            
            if self.div_apu == 7 { // Envelope step  
                self.ch1.enveloppe_tick();
                self.ch2.enveloppe_tick();
                self.ch4.enveloppe_tick();
            }
        }

        self.ch1.tick();
        self.ch2.tick();
        self.ch3.tick();

        let ch1_output = self.ch1.output();
        let ch2_output = self.ch2.output();
        let ch3_output = self.ch3.output();
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

        // Normalise for the 4 channels
        left /= 4.0;
        right /= 4.0;

        // Clamp for safety
        left = left.clamp(-1.0, 1.0);
        right = right.clamp(-1.0, 1.0);

        self.output_buffer.push((left, right));

        if self.output_buffer.len() > 1024 {
            self.output_buffer.clear();
        }

        if self.sample_timer.tick() && self.audio_enabled() {
            let (output_l, output_r) = self.filter_audio();

            while let Err(_) = self.sender.try_push((output_l, output_r)) {
                continue
            }

            self.output_buffer.clear();
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        if (self.audio_master_ctrl & 0x80) == 0 && address <= 0xFF25 {
            return; // Ignore writes when APU is off
        }

        match address {
            0xFF10..0xFF15 => self.ch1.write(address - 0xFF10, value),

            0xFF15 => (),

            0xFF16..0xFF1A => self.ch2.write(address - 0xFF15, value),

            0xFF1A..0xFF1F => self.ch3.write(address, value),

            0xFF1F => (),

            0xFF20..0xFF24 => self.ch4.write(address, value),

            0xFF24 => self.master_vol = value,
            0xFF25 => self.sound_panning = value,
            0xFF26 => self.write_nr52(value),

            0xFF30..0xFF40 => self.ch3.write(address, value), // Wave RAM
            
            _ => (), //println!("Unimplemented audio register {address:X}")
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF10..0xFF15 => self.ch1.read(address - 0xFF10),

            0xFF15 => 0xFF, // Unused

            0xFF16..0xFF1A => self.ch2.read(address - 0xFF15),

            0xFF1A..0xFF1F => self.ch3.read(address),

            0xFF1F => 0xFF, // Unused

            0xFF20..0xFF24 => self.ch4.read(address),

            0xFF24 => self.master_vol,
            0xFF25 => self.sound_panning,
            0xFF26 => {
                let mut value = self.audio_master_ctrl & 0x80;      // Keep power bit
                if self.ch1.is_enabled() { value |= 0x01; }
                if self.ch2.is_enabled() { value |= 0x02; }
                if self.ch3.is_enabled()   { value |= 0x04; }
                if self.ch4.is_enabled()  { value |= 0x08; }
                value | 0x70  // Apply mask $70
            },

            0xFF27..=0xFF2F => 0xFF, // Unused

            0xFF30..0xFF40 => self.ch3.read(address), // Wave RAM

            _ => {
                println!("Reading register {address:X}");
                panic!()
            }
        }
    }

    fn write_nr52(&mut self, value: u8) {
        let was_on = self.audio_master_ctrl & 0x80 != 0;
        let now_on = value & 0x80 != 0;
        
        self.audio_master_ctrl = value & 0x80; // Only bit 7 is writable
        
        if was_on && !now_on {
            // Powering off: clear all registers except NR52 bit 7 and wave RAM
            self.ch1.power_off();
            self.ch2.power_off();
            self.ch3.power_off();
            self.ch4.power_off();
            self.master_vol = 0;
            self.sound_panning = 0;
        }
    }

    fn filter_audio(&mut self) -> (f32, f32) {
        let n = self.output_buffer.len() as f32;
        let (mut left, mut right) = self.output_buffer.iter()
            .fold((0f32, 0f32), |acc, &(l, r)| (acc.0 + l, acc.1 + r));
        left /= n;
        right /= n;

        (left, right)                
    }

    fn audio_enabled(&self) -> bool {
        self.audio_master_ctrl & 0b10000000 != 0
    }
}