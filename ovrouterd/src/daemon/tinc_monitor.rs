use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use crate::http_server_client::Client;
use crate::domain::Info;
use crate::daemon::DaemonEvent;
use crate::tinc_manager::TincOperator;
use std::sync::mpsc::Receiver;

const TINC_FREQUENCY: u32 = 3;

pub struct TincMonitor {
    tinc: TincOperator,
}


impl TincMonitor {
    pub fn new(
        mut tinc:                  TincOperator,
    ) -> Self {
        tinc.start_tinc();
        TincMonitor {
            tinc,
        }
    }

    pub fn spawn(self) {
        thread::spawn(||self.run());
    }

    fn run(mut self) {
        let timeout_secs: u32 = TINC_FREQUENCY;
        loop {
            let start = Instant::now();
            self.exec_tinc_check();
            if let Some(remaining) =
            Duration::from_secs(timeout_secs.into()).checked_sub(start.elapsed()) {
                thread::sleep(remaining);
            }
        }
    }

    fn exec_tinc_check(&mut self) {
        trace!("check_tinc_status");
        if let Ok(_) = self.tinc.check_tinc_status() {
            return;
        }
        else {
            let mut i = 1;
            loop {
                if let Ok(_) = self.tinc.restart_tinc() {
                    return;
                }
                else {
                    error!("Restart tinc failed, try again after {} secs.", i * 5);
                    thread::sleep(Duration::from_secs(i * 5));
                    if i < 12 {
                        i += 1;
                    }
                }
            }
        }
    }
}