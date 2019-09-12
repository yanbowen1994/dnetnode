extern crate simple_signal;

use self::simple_signal::Signal;
use std::io;
use std::sync::mpsc::Sender;
use crate::daemon::DaemonEvent;

pub fn set_shutdown_signal_handler(tx: Sender<DaemonEvent>) -> Result<(), io::Error> {
    simple_signal::set_handler(&[Signal::Term, Signal::Int], move |s| {
        log::info!("Process received signal: {:?}", s);
        let _ = tx.send(DaemonEvent::ShutDown);
    });
    Ok(())
}