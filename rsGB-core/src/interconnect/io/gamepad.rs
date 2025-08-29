use crate::Button;

#[derive(Debug, Default)]
struct GamepadState {
    start: bool,
    select: bool,
    a: bool,
    b: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

#[derive(Debug, Default)]
pub struct Gamepad {
    button_select: bool,
    direction_select: bool,
    gamepad_state: GamepadState,
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
            if self.gamepad_state.start {
                output &= !(1 << 3);
            }
            if self.gamepad_state.select {
                output &= !(1 << 2);
            }
            if self.gamepad_state.a {
                output &= !(1 << 0);
            }
            if self.gamepad_state.b {
                output &= !(1 << 1);
            }
        }

        if !self.dir_sel() {
            if self.gamepad_state.left {
                output &= !(1 << 1);
            }
            if self.gamepad_state.right {
                output &= !(1 << 0);
            }
            if self.gamepad_state.up {
                output &= !(1 << 2);
            }
            if self.gamepad_state.down {
                output &= !(1 << 3);
            }
        }

        output
    }

    pub fn update_button(&mut self, button: Button, value: bool) {
        match button {
            Button::A => self.gamepad_state.a = value,
            Button::B => self.gamepad_state.b = value,
            Button::START => self.gamepad_state.start = value,
            Button::SELECT => self.gamepad_state.select = value,
            Button::UP => self.gamepad_state.up = value,
            Button::DOWN => self.gamepad_state.down = value,
            Button::LEFT => self.gamepad_state.left = value,
            Button::RIGHT => self.gamepad_state.right = value
        }
    }
}
