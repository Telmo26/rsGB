use std::{cell::RefCell, fs::File, io::{Read, Write}, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

pub struct RTC {
    live: RefCell<RtcState>,

    latched: RtcState,
    latched_valid: bool,
    last_update: RefCell<u64>,
}

impl Default for RTC {
    fn default() -> Self {
        Self {
            live: RefCell::new(RtcState::default()),
            latched: RtcState::default(),
            latched_valid: false,
            last_update: RefCell::new(current_unix_time()),
        }
    }
}

impl RTC {
    pub fn read(&self, reg: u8) -> u8 {
        self.update();

        let state = if self.latched_valid {
            &self.latched
        } else {
            &self.live.borrow()
        };

        match reg {
            0x08 => state.s,
            0x09 => state.m,
            0x0A => state.h,
            0x0B => state.dl,
            0x0C => state.dh,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, reg: u8, value: u8) {
        self.update();

        let live = self.live.get_mut();

        match reg {
            0x08 => live.s = value % 60,
            0x09 => live.m = value % 60,
            0x0A => live.h = value % 24,
            0x0B => live.dl = value,
            0x0C => live.dh = value & 0xC1, // keep only valid bits
            _ => {}
        }
    }

    pub fn latch(&mut self) {
        self.update();

        self.latched = self.live.borrow().clone();
        self.latched_valid = true;
    }

    pub fn save(&self, save_path: &PathBuf) {
        self.update();
        let mut rtc_path = save_path.clone();
        rtc_path.set_extension("rtc");

        let rtc_values = self.live.borrow().values();
        let last_update = self.last_update.borrow().clone().to_le_bytes();

        let mut file = File::create(rtc_path).expect("Unable to update RTC save");
        file.write_all(&rtc_values).unwrap();
        file.write(&last_update).unwrap();

    }

    pub fn load(&mut self, save_path: &PathBuf) {
        let mut rtc_path = save_path.clone();
        rtc_path.set_extension("rtc");
        
        if let Ok(mut file) = File::open(&rtc_path) {
            let mut rtc_values = [0; 5];
            file.read_exact(&mut rtc_values).unwrap();

            let mut timestamp_buffer = [0; 8];
            file.read_exact(&mut timestamp_buffer).unwrap();

            self.live.replace(RtcState::new(&rtc_values));
            self.last_update.replace(u64::from_le_bytes(timestamp_buffer));
        }

        self.update();
    }
}

impl RTC {
    fn update(&self) {
        let now = current_unix_time();
        let delta = self.last_update.borrow().abs_diff(now);

        self.last_update.replace(now);

        if delta <= 0 || self.live.borrow().halted() {
            return;
        }

        self.advance_seconds(delta as u64);
    }

    fn advance_seconds(&self, mut secs: u64) {
        while secs > 0 {
            let step = secs.min(60);
            self.tick(step as u32);
            secs -= step;
        }
    }

    fn tick(&self, secs: u32) {
        let mut live = self.live.borrow_mut();
        live.s += secs as u8;

        if live.s >= 60 {
            live.s %= 60;
            live.m += 1;

            if live.m == 60 {
                live.m = 0;
                live.h += 1;

                if live.h == 24 {
                    live.h = 0;

                    let mut day = live.day() + 1;
                    if day == 512 {
                        day = 0;
                        live.set_carry();
                    }

                    live.set_day(day);
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct RtcState {
    s: u8,
    m: u8,
    h: u8,
    dl: u8,
    dh: u8,
}

impl RtcState {
    fn new(values: &[u8]) -> RtcState {
        assert!(values.len() == 5);

        RtcState { 
            s: values[0], 
            m: values[1], 
            h: values[2], 
            dl: values[3], 
            dh: values[4] 
        }
    }

    fn day(&self) -> u16 {
        ((self.dh as u16 & 0x01) << 8) | self.dl as u16
    }

    fn set_day(&mut self, day: u16) {
        self.dl = (day & 0xFF) as u8;
        self.dh = (self.dh & !0x01) | ((day >> 8) as u8 & 0x01);
    }

    fn halted(&self) -> bool {
        self.dh & 0x40 != 0
    }

    fn set_carry(&mut self) {
        self.dh |= 0x80;
    }

    fn values(&self) -> [u8; 5] {
        [self.s, self.m, self.h, self.dl, self.dh]
    }
}

fn current_unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}