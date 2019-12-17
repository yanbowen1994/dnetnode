use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc;
use std::sync::mpsc::Sender;

use dnet_types::response::Response;
use dnet_types::status::TunnelState;
use tinc_plugin::TincOperatorError;

use crate::tinc_manager::TincOperator;
use crate::traits::TunnelTrait;
use crate::daemon::{DaemonEvent, TunnelCommand};
use crate::info::get_mut_info;

pub type Result<T> = std::result::Result<T, TincOperatorError>;

const TINC_FREQUENCY: u32 = 5;

pub struct TincMonitor {
    tunnel_command_rx:      mpsc::Receiver<(TunnelCommand, mpsc::Sender<Response>)>,
    inner_cmd_tx:           mpsc::Sender<InnerStatus>,
}

impl TunnelTrait for TincMonitor {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> (Self, mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>) {
        Self::init_tinc(daemon_event_tx);

        let (tunnel_command_tx, tunnel_command_rx) = mpsc::channel();

        let inner_cmd_tx = MonitorInner::new(
            tunnel_command_tx.clone()
        );

        let tinc_monitor = TincMonitor {
            tunnel_command_rx,
            inner_cmd_tx,
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
                    let res = Self::connect();
                    let _ = self.inner_cmd_tx.send(InnerStatus::Start);
                    let _ = res_tx.send(res);
                }
                TunnelCommand::Disconnect => {
                    let res = self.disconnect();
                    let _ = res_tx.send(res);
                }
                TunnelCommand::Reconnect => {
                    Self::reconnect(res_tx);
                }
                TunnelCommand::Connected => {
                    let inner_cmd_tx = self.inner_cmd_tx.clone();
                    thread::spawn(move || {
                        let _ = inner_cmd_tx.send(InnerStatus::Start);
                        let _ = res_tx.send(Response::success());
                    });
                }
                TunnelCommand::Disconnected => {
                    let _ = res_tx.send(Response::success());
                }
            }
        }
    }

    fn init_tinc(daemon_event_tx: mpsc::Sender<DaemonEvent>) {
        let tinc = TincOperator::new();
        // 初始化tinc操作
        // 监测tinc pub key 不存在或生成时间超过一个月，将生成tinc pub key
        info!("check_pub_key");
        tinc.init()
            .map_err(|e|
                daemon_event_tx.send(DaemonEvent::TunnelInitFailed(e.to_string()))
            )
            .unwrap_or(());
    }

    fn connect() -> Response {
        let mut info = get_mut_info().lock().unwrap();
        let res =
            if info.status.tunnel == TunnelState::Disconnected
                || info.status.tunnel == TunnelState::Disconnecting {
                info.status.tunnel = TunnelState::Connecting;
                std::mem::drop(info);

                match TincOperator::new().start_tinc() {
                    Ok(_) => {
                        info!("tinc_monitor start tinc");
                        Response::success()
                    },
                    Err(err) => {
                        error!("connect {:?}", err);
                        Response::internal_error().set_msg(format!("{:?}", err))
                    },
                }
            }
            else {
                std::mem::drop(info);
                Response::success()
            };
        res
    }

    fn reconnect(res_tx: Sender<Response>) {
        std::thread::spawn(move|| {
            let res = match TincOperator::new().restart_tinc() {
                Ok(_) => {
                    info!("tinc_monitor restart tinc");
                    Response::success()
                },
                Err(err) => {
                    error!("reconnect {:?}", err);
                    Response::internal_error().set_msg(format!("{:?}", err))
                },
            };
            let _ = res_tx.send(res);
        });
    }

    fn disconnect(&self) -> Response {
        let mut info = get_mut_info().lock().unwrap();
        info.status.tunnel = TunnelState::Disconnecting;
        std::mem::drop(info);
        let res =
            if let Err(err) = TincOperator::new().stop_tinc() {
                Response::internal_error().set_msg(err.to_string())
            }
            else {
                match self.inner_cmd_tx.send(InnerStatus::Stop) {
                    Ok(_) => {
                        Response::success()
                    }
                    Err(_) => {
                        Response::internal_error().set_msg("Can't stop tinc check.".to_string())
                    }
                }
            };

        info!("disconnect {:?}", res);
        res
    }
}

#[derive(Debug, PartialEq)]
enum InnerStatus {
    Start,
    Stop
}

struct MonitorInner {
    tunnel_command_tx:   mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>,
    start_stop_sign_rx:  mpsc::Receiver<InnerStatus>,
}

impl MonitorInner {
    fn new(tunnel_command_tx: mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>
    ) -> mpsc::Sender<InnerStatus> {
        let (inner_cmd_tx, start_stop_sign_rx) = mpsc::channel();

        thread::spawn(|| {
            Self {
                tunnel_command_tx,
                start_stop_sign_rx,
            }.run();
        });

        inner_cmd_tx
    }

    fn run(&mut self) {
        let mut status = InnerStatus::Stop;
        let mut check_time = Instant::now();
        info!("tinc check start.");
        loop {
            if let Ok(res_status) = self.start_stop_sign_rx.try_recv() {
                info!("tinc check cmd:{:?}", res_status);
                status = res_status;
            }
            if status == InnerStatus::Start {
                if Instant::now() - check_time > Duration::from_secs(TINC_FREQUENCY.into()) {
                    debug!("exec_tinc_check");
                    if let Ok(_) = self.exec_tinc_check() {
                        get_mut_info().lock().unwrap().status.tunnel = TunnelState::Connected;
                    }
                    else {
                        let mut info = get_mut_info().lock().unwrap();
                        info.status.tunnel = TunnelState::Disconnected;
                        std::mem::drop(info);
                        let (tx, _) = mpsc::channel();
                        let _ = self.tunnel_command_tx.send((TunnelCommand::Reconnect, tx));
                    }
                    check_time = Instant::now();
                }
            }
            thread::sleep(Duration::from_millis(1500));
        }
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
}