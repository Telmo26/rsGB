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

#[derive(Debug, Clone, Copy)]
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