extern crate tinc_plugin;
use tinc_plugin::control::dump_connections;
use tinc_plugin::listener::spawn;

use std::thread;
use std::sync::mpsc::Receiver;
use tinc_plugin::EventType;
use std::time::Duration;

fn test() -> Receiver<EventType> {
    let a = spawn();
    a
}

fn main() {
    let a = test();
    thread::sleep(Duration::from_secs(3));
    loop {
        match a.recv() {
            Ok(event) => println!("{:?}", event),
            Err(_e) => (),
        }
    }
}
