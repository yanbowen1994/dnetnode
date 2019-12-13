use std::thread;
use std::time::{Duration, Instant};
use std::sync::{mpsc, Mutex, Arc};

use dnet_types::response::Response;
use tinc_plugin::TincOperatorError;

use crate::tinc_manager::TincOperator;
use crate::traits::TunnelTrait;
use crate::daemon::{DaemonEvent, TunnelCommand};
use crate::info::get_info;
use dnet_types::status::TunnelState;

pub type Result<T> = std::result::Result<T, TincOperatorError>;

const TINC_FREQUENCY: u32 = 5;

static mut EL: *mut MonitorInner = 0 as *mut _;

pub struct TincMonitor {
    tunnel_command_rx:      mpsc::Receiver<(TunnelCommand, mpsc::Sender<Response>)>,
}

impl TunnelTrait for TincMonitor {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> (Self, mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>) {
        let (tunnel_command_tx, tunnel_command_rx) = mpsc::channel();

        MonitorInner::new(daemon_event_tx.clone());

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
            info!("TincMonitor event {:?}", event);
            match event {
                TunnelCommand::Connect => {
                    let tunnel_status = get_info().lock().unwrap().status.tunnel.clone();
                    let res =
                        if tunnel_status == TunnelState::Disconnected
                            || tunnel_status == TunnelState::Disconnecting {
                            let inner = get_monitor_inner();

                            match inner.connect() {
                                Ok(_) => {
                                    thread::spawn(move || {
                                        inner.run();
                                    });
                                    Response::success()
                                },
                                Err(err) => Response::internal_error().set_msg(format!("{:?}", err)),
                            }
                        }
                        else {
                            Response::success()
                        };

                    let _ = res_tx.send(res);
                }
                TunnelCommand::Disconnect => {
                    let tunnel_status = get_info().lock().unwrap().status.tunnel.clone();
                    let res =
                        if tunnel_status == TunnelState::Connected
                            || tunnel_status == TunnelState::Connecting {
                            let inner = get_monitor_inner();
                            match inner.disconnect() {
                                Ok(_) => Response::success(),
                                Err(err) => Response::internal_error().set_msg(format!("{:?}", err)),
                            }
                        }
                        else {
                            Response::success()
                        };

                    let _ = res_tx.send(res);
                }
                TunnelCommand::Reconnect => {
                    let inner = get_monitor_inner();
                    let res = match inner.reconnect() {
                        Ok(_) => {
                            Response::success()
                        },
                        Err(err) => Response::internal_error().set_msg(format!("{:?}", err)),
                    };
                    let _ = res_tx.send(res);
                }
                TunnelCommand::Connected => {
                    let _ = res_tx.send(Response::success());
                }
                TunnelCommand::Disconnected => {
                    ()
                }
            }
        }
    }
}

struct MonitorInner {
    stop_sign_tx:                       mpsc::Sender<mpsc::Sender<bool>>,
    stop_sign_rx:                       mpsc::Receiver<mpsc::Sender<bool>>,
    is_running:                         Arc<Mutex<bool>>,
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
            stop_sign_tx:       tx,
            stop_sign_rx:       rx,
            is_running:         Arc::new(Mutex::new(false)),
        };

        unsafe {
            EL = Box::into_raw(Box::new(inner));
        };
    }

    fn connect(&mut self) -> Result<()> {
        TincOperator::new().start_tinc()?;
        info!("tinc_monitor start tinc");
        Ok(())
    }

    fn run(&mut self) {
        let mut is_running = self.is_running.lock().unwrap();
        if *is_running {
            return;
        }
        else {
            *is_running = true;
        }
        std::mem::drop(is_running);

        let mut check_time = Instant::now();

        info!("tinc check start.");

        loop {
            if let Ok(res_tx) = self.stop_sign_rx.try_recv() {
                let _ = res_tx.send(true);
                let mut is_running = self.is_running.lock().unwrap();
                *is_running = false;
                break
            }

            if Instant::now() - check_time > Duration::from_secs(TINC_FREQUENCY.into()) {
                debug!("exec_tinc_check");
                if let Err(_) = self.exec_tinc_check() {
                    self.exec_restart();
                }
                check_time = Instant::now();
            }
            thread::sleep(Duration::from_millis(1500));
        }
        info!("tinc check stop.");
    }

    fn disconnect(&mut self) -> Result<()> {
        if *self.is_running.lock().unwrap() {
            let (tx, rx) = mpsc::channel();
            if let Err(err) = self.stop_sign_tx.send(tx) {
                error!("disconnect {:?}", err);
            }
            else {
                let _ = rx.recv_timeout(Duration::from_secs(5));
            }
        }
        let mut tinc = TincOperator::new();
        tinc.stop_tinc()?;
        info!("tinc_monitor stop tinc");
        Ok(())
    }

    fn reconnect(&mut self) -> Result<()> {
        if *self.is_running.lock().unwrap() {
            let (tx, rx) = mpsc::channel();
            match self.stop_sign_tx.send(tx) {
                Ok(_) => {
                    let _ = rx.recv_timeout(Duration::from_secs(5));
                    ()
                },
                Err(_) => return Err(TincOperatorError::StopTincError),
            }
        }
        self.connect()?;
        info!("tinc_monitor restart tinc success");
        Ok(())
    }

    fn exec_tinc_check(&mut self) -> Result<()> {
        let mut tinc = TincOperator::new();

        match tinc.check_tinc_status() {
            Ok(_) => {
                debug!("check tinc process: tinc exist.");
                return Ok(());
            }

            Err(TincOperatorError::OutOfMemory) => {
                error!("check tinc process: tinc out of memory.");
                return Err(TincOperatorError::OutOfMemory);
            }

            Err(TincOperatorError::TincNotExist) => {
                error!("check tinc process: tinc not exist.");
                return Err(TincOperatorError::TincNotExist);
            }

            Err(err) => {
                error!("check tinc process: check failed. error:{:?}", err);
                return Err(err);
            }
        }
    }

    fn exec_restart(&mut self) {
        let mut tinc = TincOperator::new();
        let mut i = 1;
        loop {
            let result;
            {
                result = tinc.restart_tinc();
            }
            match result {
                Ok(_) => {
                    info!("check tinc process: execute restart tinc.");
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