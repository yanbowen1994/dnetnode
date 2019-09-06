use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc;

use crate::tinc_manager::TincOperator;
use tinc_plugin::TincOperatorError;
use common_core::traits::TunnelTrait;
use common_core::daemon::DaemonEvent;

const TINC_FREQUENCY: u32 = 5;

pub struct TincMonitor {
    tinc: TincOperator,
    daemon_event_tx: mpsc::Sender<DaemonEvent>,
}

impl TunnelTrait for TincMonitor {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> Self {
        let tinc = TincOperator::new();

        // 初始化tinc操作
        tinc.create_tinc_dirs()
            .map_err(|e|
                daemon_event_tx.send(DaemonEvent::TunnelInitFailed(e.to_string()))
            )
            .unwrap_or(());

        // 监测tinc pub key 不存在或生成时间超过一个月，将生成tinc pub key
        info!("check_pub_key");
        if !tinc.check_pub_key() {
            tinc.create_pub_key()
                .map_err(|e|
                    daemon_event_tx.send(DaemonEvent::TunnelInitFailed(e.to_string()))
                )
                .unwrap_or(());
        }

        TincMonitor {
            tinc,
            daemon_event_tx,
        }
    }

    fn start_monitor(self) {
        thread::spawn(move ||self.run());
    }
}


impl TincMonitor {
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
            return ();
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