use bitflags::bitflags;

#[macro_export]
macro_rules! NO_IMPL {
    () => {{
        panic!("NOT YET IMPLEMENTED");
    }};
}

/// This function sets the n-th bit of 'a' 
/// to the value given in 'on'
pub fn bit_set(a: &mut u8, n: u8, on: bool) {
    if on {
        *a |= 1 << n;
    } else {
        *a &= !(1 << n);
    }
}

pub const BIT_IGNORE: u8 = 2;

pub type VRAM = [u8; 0x2000];

pub const TICKS_PER_SAMPLE: u64 = 95;

#[derive(Debug, Clone, Copy)]
pub enum ColorMode {
    RGBA,
    ARGB,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Button {
    A,
    B,
    START,
    SELECT,
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

impl Button {
    fn flag(&self) -> ButtonFlag {
        match self {
            Button::A       => ButtonFlag::A,
            Button::B       => ButtonFlag::B,
            Button::START   => ButtonFlag::START,
            Button::SELECT  => ButtonFlag::SELECT,
            Button::UP      => ButtonFlag::UP,
            Button::DOWN    => ButtonFlag::DOWN,
            Button::LEFT    => ButtonFlag::LEFT,
            Button::RIGHT   => ButtonFlag::RIGHT
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Default)]
    pub struct ButtonFlag: u8 {
        const RIGHT  = 0b0000_0001;
        const LEFT   = 0b0000_0010;
        const UP     = 0b0000_0100;
        const DOWN   = 0b0000_1000;
        const A      = 0b0001_0000;
        const B      = 0b0010_0000;
        const SELECT = 0b0100_0000;
        const START  = 0b1000_0000;
    }
}

#[derive(Debug, Default)]
pub struct InputState {
    buttons: ButtonFlag,
}

impl InputState {
    pub fn update(&mut self, button: impl Into<Button>, pressed: bool) {
        let button = button.into();
        self.buttons.set(button.flag(), pressed);
    }

    pub fn pressed(&self, button: impl Into<Button>) -> bool {
        self.buttons.contains(button.into().flag())
    }

    pub fn clear(&mut self) {
        self.buttons = ButtonFlag::empty();
    }

    pub fn from_bits(bits: u8) -> InputState {
        InputState { buttons: ButtonFlag::from_bits_truncate(bits) }
    }

    pub fn bits(&self) -> u8 {
        self.buttons.bits()
    }
}