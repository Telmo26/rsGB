use std::env;

use rs_gb::emu_run;

fn main() {
    let args: Vec<String> = env::args()
        .collect();
    emu_run(args)
}
