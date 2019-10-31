#[cfg(unix)]
extern crate simple_signal;
#[cfg(unix)]
use self::simple_signal::Signal;
use std::io;
use std::sync::mpsc::Sender;
use crate::daemon::DaemonEvent;
#[cfg(windows)]
extern crate ctrlc;

#[cfg(unix)]
pub fn set_shutdown_signal_handler(tx: Sender<DaemonEvent>) -> Result<(), io::Error> {
    simple_signal::set_handler(&[Signal::Term, Signal::Int], move |s| {
        log::info!("Process received signal: {:?}", s);
        let _ = tx.send(DaemonEvent::ShutDown);
    });
    Ok(())
}

#[cfg(windows)]
pub fn set_shutdown_signal_handler(tx: Sender<DaemonEvent>) -> Result<(), io::Error> {
    ctrlc::set_handler(move || {
        log::debug!("Process received Ctrl-c");
        let _ = tx.send(DaemonEvent::ShutDown);
    })
        .map_err(|_|io::Error::new(io::ErrorKind::NotFound, "123"))
}
