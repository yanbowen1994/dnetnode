use std::thread;
use std::time::{Duration, Instant};
use std::sync::{mpsc, Mutex, Arc};

use crate::tinc_manager::TincOperator;
use crate::traits::TunnelTrait;
use crate::daemon::{DaemonEvent, TunnelCommand};
use crate::info::{Info, get_info};
use std::sync::mpsc::{Receiver, Sender};
use std::borrow::BorrowMut;

const TINC_FREQUENCY: u32 = 5;

static mut EL: *mut MonitorInner = 0 as *mut _;

pub struct TincMonitor {
    connect_cmd_mutex:      Arc<Mutex<bool>>,
    daemon_event_tx:        mpsc::Sender<DaemonEvent>,
    tunnel_command_rx:      mpsc::Receiver<TunnelCommand>,
}

impl TunnelTrait for TincMonitor {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> (Self, mpsc::Sender<TunnelCommand>) {
        let (tunnel_command_tx, tunnel_command_rx) = mpsc::channel();

        Arc::new(MonitorInner::new(daemon_event_tx.clone()));

        let tinc_monitor = TincMonitor {
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
                    let mut inner = get_monitor_inner();
                    thread::spawn(move || inner.connect());
                }
                TunnelCommand::Disconnect => {
                    if let Ok(mut connect_cmd_mutex) = self.connect_cmd_mutex.lock() {
                        *connect_cmd_mutex = false;
                    }
                    let mut inner = get_monitor_inner();
                    inner.disconnect();
                }
            }
        }
    }
}

struct MonitorInner {
    daemon_event_tx:    Mutex<Sender<DaemonEvent>>,
    stop_sign:          Mutex<u32>,
}

impl MonitorInner {
    fn new(daemon_event_tx: Sender<DaemonEvent>) {

        let tinc = TincOperator::new();
        // 初始化tinc操作
        // 监测tinc pub key 不存在或生成时间超过一个月，将生成tinc pub key
        info!("check_pub_key");
        tinc.init()
            .map_err(|e|
                daemon_event_tx.send(DaemonEvent::TunnelInitFailed(e.to_string()))
            )
            .unwrap_or(());
        let inner = Self {
            daemon_event_tx:    Mutex::new(daemon_event_tx),
            stop_sign:          Mutex::new(0),
        };

        unsafe {
            EL = Box::into_raw(Box::new(inner));
        };

    }

    fn connect(&mut self) {
        *self.stop_sign.lock().unwrap() == 0;
        let mut tinc = TincOperator::new();
        {
            let _ = tinc.start_tinc()
                .map_err(|e|
                    self.daemon_event_tx.lock().unwrap()
                        .send(DaemonEvent::TunnelInitFailed(e.to_string())));
        }
        loop {
            if *self.stop_sign.lock().unwrap() == 1 {
                break
            }
            let start = Instant::now();
            self.exec_tinc_check();

            if let Some(remaining) =
            Duration::from_secs(TINC_FREQUENCY.into()).checked_sub(start.elapsed()) {
                thread::sleep(remaining);
            }
        }
    }

    fn disconnect(&mut self) {
        *self.stop_sign.lock().unwrap() = 1;
        let mut tinc = TincOperator::new();
        tinc.stop_tinc();
    }

    fn exec_tinc_check(&mut self) {
        let mut tinc = TincOperator::new();
        {
            if let Ok(_) = tinc.check_tinc_status() {
                trace!("check tinc process: tinc exist.");
                return;
            }
        }
        error!("check tinc process: tinc not exist.");
        let mut i = 1;
        loop {
            let result;
            {
                result = tinc.restart_tinc();
            }
            match result {
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
        }
    }
}

fn get_monitor_inner() ->  &'static mut MonitorInner {
    unsafe {
        if EL == 0 as *mut _ {
            panic!("Get settings instance, before init");
        }
        &mut *EL
    }
}