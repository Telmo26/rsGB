#[macro_export]
macro_rules! NO_IMPL {
    () => {{
        eprintln!("NOT YET IMPLEMENTED");
        std::process::exit(-5);
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