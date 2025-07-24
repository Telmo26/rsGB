use std::env;

use rs_gb::run;

fn main() {
    let args: Vec<String> = env::args().collect();
    run(args)
}
