use crate::{Button, InputState};

#[derive(Debug, Default)]
pub struct Gamepad {
    button_select: bool,
    direction_select: bool,
    pub(super) gamepad_state: InputState,
}

impl Gamepad {
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
        let mut output: u8 = 0xCF;
        if !self.button_sel() {
            if self.gamepad_state.pressed(Button::START) {
                output &= !(1 << 3);
            }
            if self.gamepad_state.pressed(Button::SELECT) {
                output &= !(1 << 2);
            }
            if self.gamepad_state.pressed(Button::A) {
                output &= !(1 << 0);
            }
            if self.gamepad_state.pressed(Button::B) {
                output &= !(1 << 1);
            }
        }

        if !self.dir_sel() {
            if self.gamepad_state.pressed(Button::LEFT) {
                output &= !(1 << 1);
            }
            if self.gamepad_state.pressed(Button::RIGHT) {
                output &= !(1 << 0);
            }
            if self.gamepad_state.pressed(Button::UP) {
                output &= !(1 << 2);
            }
            if self.gamepad_state.pressed(Button::DOWN) {
                output &= !(1 << 3);
            }
        }

        output
    }
}
