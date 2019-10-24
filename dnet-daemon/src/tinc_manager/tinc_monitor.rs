use std::thread;
use std::time::{Duration, Instant};
use std::sync::{mpsc, Mutex, Arc};

use dnet_types::response::Response;
use tinc_plugin::TincOperatorError;

use crate::tinc_manager::TincOperator;
use crate::traits::TunnelTrait;
use crate::daemon::{DaemonEvent, TunnelCommand};

pub type Result<T> = std::result::Result<T, TincOperatorError>;

const TINC_FREQUENCY: u32 = 5;

static mut EL: *mut MonitorInner = 0 as *mut _;

pub struct TincMonitor {
    tunnel_command_rx:      mpsc::Receiver<(TunnelCommand, mpsc::Sender<Response>)>,
}

impl TunnelTrait for TincMonitor {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> (Self, mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>) {
        let (tunnel_command_tx, tunnel_command_rx) = mpsc::channel();

        Arc::new(MonitorInner::new(daemon_event_tx.clone()));

        let tinc_monitor = TincMonitor {
            tunnel_command_rx,
        };

        return (tinc_monitor, tunnel_command_tx)
    }

    fn start_monitor(self) {
        thread::spawn(move ||self.run());
    }
}

impl TincMonitor {
    fn run(self) {
        while let Ok((event, res_tx)) = self.tunnel_command_rx.recv() {
            match event {
                TunnelCommand::Connect => {
                    let inner = get_monitor_inner();
                    let res = match inner.connect() {
                        Ok(_) => Response::success(),
                        Err(err) => Response::internal_error().set_msg(format!("{:?}", err)),
                    };

                    thread::spawn(move || {
                        // wait tunnel TODO use ipc get tunnel start. tinc -> tinc-up -> ipc -> tinc-monitor
                        thread::sleep(Duration::from_secs(3));
                        inner.run();
                    });

                    let _ = res_tx.send(res);
                }
                TunnelCommand::Disconnect => {
                    let inner = get_monitor_inner();
                    let res = match inner.disconnect() {
                        Ok(_) => Response::success(),
                        Err(err) => Response::internal_error().set_msg(format!("{:?}", err)),
                    };
                    let _ = res_tx.send(res);
                }
                TunnelCommand::Reconnect => {
                    let inner = get_monitor_inner();
                    let res = match inner.reconnect() {
                        Ok(_) => Response::success(),
                        Err(err) => Response::internal_error().set_msg(format!("{:?}", err)),
                    };
                    let _ = res_tx.send(res);
                }
            }
        }
    }
}

struct MonitorInner {
    stop_sign_tx:          mpsc::Sender<mpsc::Sender<bool>>,
    stop_sign_rx:          mpsc::Receiver<mpsc::Sender<bool>>,
    is_tunnel_connect:        Arc<Mutex<bool>>,
}

impl MonitorInner {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) {
        let tinc = TincOperator::new();
        // 初始化tinc操作
        // 监测tinc pub key 不存在或生成时间超过一个月，将生成tinc pub key
        info!("check_pub_key");
        tinc.init()
            .map_err(|e|
                daemon_event_tx.send(DaemonEvent::TunnelInitFailed(e.to_string()))
            )
            .unwrap_or(());

        let (tx, rx) = mpsc::channel();

        let inner = Self {
            stop_sign_tx: tx,
            stop_sign_rx: rx,
            is_tunnel_connect: Arc::new(Mutex::new(false)),
        };

        unsafe {
            EL = Box::into_raw(Box::new(inner));
        };

    }

    fn connect(&mut self) -> Result<()> {
        TincOperator::new().start_tinc()?;
        info!("tinc_monitor start tinc");
        *self.is_tunnel_connect.lock().unwrap() = true;

        Ok(())
    }

    fn run(&mut self) {
        loop {
            if let Ok(res_tx) = self.stop_sign_rx.try_recv() {
                let _ = res_tx.send(true);
            }
            let start = Instant::now();
            self.exec_tinc_check();

            if let Some(remaining) =
            Duration::from_secs(TINC_FREQUENCY.into()).checked_sub(start.elapsed()) {
                thread::sleep(remaining);
            }
        }
    }

    fn disconnect(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        if let Err(err) = self.stop_sign_tx.send(tx) {
            let mut tinc = TincOperator::new();
            tinc.stop_tinc()?;
        }
        else {
            let _ = rx.recv_timeout(Duration::from_secs(1));
            let mut tinc = TincOperator::new();
            tinc.stop_tinc()?;
        }
        info!("tinc_monitor stop tinc");
        Ok(())
    }

    fn reconnect(&mut self) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        match self.stop_sign_tx.send(tx) {
            Ok(_) => {
                let _ = rx.recv_timeout(Duration::from_secs(1));
                ()
            },
            Err(_) => (),
        }
        let mut tinc = TincOperator::new();
        tinc.restart_tinc()?;
        info!("tinc_monitor restart tinc");
        Ok(())
    }

    fn exec_tinc_check(&mut self) {
        let mut tinc = TincOperator::new();

        match tinc.check_tinc_status() {
            Ok(_) => {
                trace!("check tinc process: tinc exist.");
                return ();
            }

            Err(TincOperatorError::OutOfMemory) => {
                error!("check tinc process: tinc out of memory.");
            }

            Err(TincOperatorError::TincNotExist) => {
                error!("check tinc process: tinc not exist.");
            }

            Err(err) => {
                error!("check tinc process: check failed. error:{:?}", err);
            }
        }

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