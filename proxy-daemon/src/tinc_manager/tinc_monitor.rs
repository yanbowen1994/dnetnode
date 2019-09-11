use std::thread;
use std::time::{Duration, Instant};
use std::sync::{mpsc, Mutex, Arc};

use crate::tinc_manager::TincOperator;
use common_core::traits::TunnelTrait;
use common_core::daemon::{DaemonEvent, TunnelCommand};

const TINC_FREQUENCY: u32 = 5;

pub struct TincMonitor {
    tinc:                   TincOperator,
    connect_cmd_mutex:      Arc<Mutex<bool>>,
    daemon_event_tx:        mpsc::Sender<DaemonEvent>,
    tunnel_command_rx:      mpsc::Receiver<TunnelCommand>,
}

impl TunnelTrait for TincMonitor {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> (Self, mpsc::Sender<TunnelCommand>) {
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

        let (tunnel_command_tx, tunnel_command_rx) = mpsc::channel();

        let tinc_monitor = TincMonitor {
            tinc,
            connect_cmd_mutex: Arc::new(Mutex::new(false)),
            daemon_event_tx,
            tunnel_command_rx,
        };

        return (tinc_monitor, tunnel_command_tx)
    }

    fn start_monitor(self) {
        thread::spawn(move ||self.run());
    }
}


impl TincMonitor {
    fn run(mut self) {
        while let Ok(event) = self.tunnel_command_rx.recv() {
            match event {
                TunnelCommand::Connect => {
                    if let Ok(mut connect_cmd_mutex) = self.connect_cmd_mutex.lock() {
                        *connect_cmd_mutex = true;
                    }
                    self.connect();
                }
                TunnelCommand::Disconnect => {
                    if let Ok(mut connect_cmd_mutex) = self.connect_cmd_mutex.lock() {
                        *connect_cmd_mutex = false;
                    }
                }
            }
        }
    }

    fn connect(&mut self) {
        let _ = self.tinc.start_tinc()
            .map_err(|e|
                self.daemon_event_tx.send(DaemonEvent::TunnelInitFailed(e.to_string())));
        loop {
            let start = Instant::now();
            self.exec_tinc_check();
            if let Some(remaining) =
            Duration::from_secs(TINC_FREQUENCY.into()).checked_sub(start.elapsed()) {
                thread::sleep(remaining);
            }
        }
    }

    fn disconnect(&mut self) {
        self.tinc.stop_tinc();
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