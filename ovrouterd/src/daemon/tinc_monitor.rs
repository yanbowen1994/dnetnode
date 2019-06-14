use std::thread;
use std::time::{Duration, Instant};
use crate::tinc_manager::TincOperator;

const TINC_FREQUENCY: u32 = 5;

pub type Result<T> = std::result::Result<T, ::tinc_manager::operator::Error>;

pub struct TincMonitor {
    tinc: TincOperator,
}


impl TincMonitor {
    pub fn new(
        mut tinc:                  TincOperator,
    ) -> Result<Self> {
        tinc.start_tinc()?;
        Ok(TincMonitor {
            tinc,
        })
    }

    pub fn spawn(self) {
        thread::spawn(||self.run());
    }

    fn run(mut self) {
        loop {
            let start = Instant::now();
            self.exec_tinc_check();
            if let Some(remaining) =
            Duration::from_secs(TINC_FREQUENCY.into()).checked_sub(start.elapsed()) {
                thread::sleep(remaining);
            }
        }
    }

    fn exec_tinc_check(&mut self) {
        if let Ok(_) = self.tinc.check_tinc_status() {
            trace!("check tinc process: tinc exist.");
            return;
        }
        error!("check tinc process: tinc not exist.");
        let mut i = 1;
        loop {
            match self.tinc.restart_tinc() {
                Ok(_) => {
                    info!("check tinc process: tinc restart finish.");
                    return;
                },
                Err(e) => {
                    error!("Restart tinc failed.\n{:?}\n try again after {} secs.", e, i * 5);
                    thread::sleep(Duration::from_secs(i * 5));
                    if i < 12 {
                        i += 1;
                    }
                }
            }
            if let Ok(_) = self.tinc.restart_tinc() {
                info!("check tinc process: tinc restart finish.");
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