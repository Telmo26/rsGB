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

impl GamepadState {
    pub fn new() -> GamepadState {
        GamepadState {
            start: false,
            select: false,
            a: false,
            b: false,
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
}

pub struct Gamepad {
    button_select: bool,
    direction_select: bool,
    controller: GamepadState,
}

impl Gamepad {
    pub fn new() -> Gamepad {
        Gamepad {
            button_select: false,
            direction_select: false,
            controller: GamepadState::new(),
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
        let mut output: u8 = 0xCF;
        if !self.button_sel() {
            if self.controller.start {
                output &= !(1 << 3);
            }
            if self.controller.select {
                output &= !(1 << 2);
            }
            if self.controller.a {
                output &= !(1 << 0);
            }
            if self.controller.b {
                output &= !(1 << 1);
            }
        }

        if !self.dir_sel() {
            if self.controller.left {
                output &= !(1 << 1);
            }
            if self.controller.right {
                output &= !(1 << 0);
            }
            if self.controller.up {
                output &= !(1 << 2);
            }
            if self.controller.down {
                output &= !(1 << 3);
            }
        }

        output
    }
}
