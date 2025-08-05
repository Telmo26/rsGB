use std::sync::{Arc, Mutex};

use crate::GamepadState;

pub struct Gamepad {
    button_select: bool,
    direction_select: bool,
    gamepad_state: Arc<Mutex<GamepadState>>,
}

impl Gamepad {
    pub fn new(gamepad_state: Arc<Mutex<GamepadState>>) -> Gamepad {
        Gamepad {
            button_select: false,
            direction_select: false,
            gamepad_state,
        }
    }

    pub fn button_sel(&self) -> bool {
        self.button_select
    }

    pub fn dir_sel(&self) -> bool {
        self.direction_select
    }

    pub fn set_sel(&mut self, value: u8) {
        self.button_select = (value & 0x20) != 0;
        self.direction_select = (value & 0x10) != 0;

    }

    pub fn get_output(&self) -> u8 {
        let gamepad_lock = self.gamepad_state.lock().unwrap();
        let mut output: u8 = 0xCF;
        if !self.button_sel() {
            if gamepad_lock.start {
                output &= !(1 << 3);
            }
            if gamepad_lock.select {
                output &= !(1 << 2);
            }
            if gamepad_lock.a {
                output &= !(1 << 0);
            }
            if gamepad_lock.b {
                output &= !(1 << 1);
            }
        }

        if !self.dir_sel() {
            if gamepad_lock.left {
                output &= !(1 << 1);
            }
            if gamepad_lock.right {
                output &= !(1 << 0);
            }
            if gamepad_lock.up {
                output &= !(1 << 2);
            }
            if gamepad_lock.down {
                output &= !(1 << 3);
            }
        }

        output
    }
}
