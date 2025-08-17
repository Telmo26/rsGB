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
    ch1_timer: Timer,
    ch1_waveform_pointer: u8,

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
            sampling_timer: Timer::new(87),

            div_apu: 0,
            
            ch1: PulseChannel::default(),
            ch1_timer: Timer::default(),
            ch1_waveform_pointer: 0,

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
        }

        self.ch2.tick();

        if self.sampling_timer.tick() {
            let ch1_output = 0.0;
            let ch2_output = self.ch2.output();
            let ch3_output = 0.0;
            let ch4_output = 0.0;

            let output = ch1_output + ch2_output + ch3_output + ch4_output;

            let _ = self.sender.try_push(output);
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF10 => self.ch1.sweep = value,
            0xFF11 => self.ch1.length_timer_duty_cycle = value,
            0xFF12 => self.ch1.volume_envelope = value,
            0xFF13 => self.ch1.period_low = value,
            0xFF14 => self.ch1.period_high_ctrl = value,

            0xFF15..0xFF1A => self.ch2.write(address - 0xFF15, value),

            0xFF24 => self.master_vol = value,
            0xFF25 => self.sound_panning = value,
            0xFF26 => self.audio_master_ctrl = value,
            
            _ => println!("Unimplemented audio register {address:X}")
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF10 => self.ch1.sweep,
            0xFF11 => self.ch1.length_timer_duty_cycle,
            0xFF12 => self.ch1.volume_envelope,
            0xFF13 => self.ch1.period_low,
            0xFF14 => self.ch1.period_high_ctrl,

            0xFF15..0xFF1A => self.ch2.read(address - 0xFF15),

            0xFF24 => self.master_vol,
            0xFF25 => self.sound_panning,
            0xFF26 => self.audio_master_ctrl,

            _ => unreachable!()
        }
    }
}