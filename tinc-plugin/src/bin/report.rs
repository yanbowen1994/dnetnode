use std::env;

use tinc_plugin::listener::start;

fn main() {
    let args: Vec<String> = env::args().collect();
    start(args);
}