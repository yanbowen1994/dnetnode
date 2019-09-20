use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use crate::daemon::DaemonEvent;
use crate::traits::RpcTrait;
use crate::settings::get_settings;
use crate::info::Info;
use crate::tinc_manager::TincOperator;

use super::RpcClient;
use crate::rpc::rpc_cmd::{RpcCmd, RpcClientCmd};
use std::sync::mpsc::Receiver;

const HEARTBEAT_FREQUENCY: u32 = 20;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Connection with conductor timeout")]
    RpcTimeout,
}

#[derive(Eq, PartialEq)]
enum RunStatus {
    NotSendHearbeat,
    SendHearbeat,
    Restart,
}

pub struct RpcMonitor {
    client:                     RpcClient,
    daemon_event_tx:            mpsc::Sender<DaemonEvent>,
    rpc_cmd_rx:                 mpsc::Receiver<RpcCmd>,
    run_status:                 RunStatus,
}

impl RpcTrait for RpcMonitor {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> mpsc::Sender<RpcCmd> {
        let (rpc_cmd_tx, rpc_cmd_rx) = mpsc::channel();

        let client = RpcClient::new();
        RpcMonitor {
            client,
            daemon_event_tx,
            rpc_cmd_rx,
            run_status: RunStatus::NotSendHearbeat,
        }.start_monitor();
        return rpc_cmd_tx;
    }
}

impl RpcMonitor {
    fn start_monitor(self) {
        // TODO async
        thread::spawn(||self.run());
    }

    fn run(mut self) {
        let timeout_secs: u32 = 500;
        loop {
            self.run_status = RunStatus::NotSendHearbeat;
            self.init();
            loop {
                if let Ok(cmd) = self.rpc_cmd_rx.try_recv() {
                    match cmd {
                        RpcCmd::Client(cmd) => {
                            match cmd {
                                RpcClientCmd::StartHeartbeat => {
                                    self.run_status = RunStatus::SendHearbeat;
                                },

                                RpcClientCmd::RestartRpcConnect => {
                                    break
                                }

                                RpcClientCmd::JoinTeam(team_id) => {
                                    self.client.join_team(team_id);
                                }
                            }
                        }
                        _ => ()
                    }
                }

                if self.run_status == RunStatus::SendHearbeat {
                    let start = Instant::now();

                    if let Err(_) = self.exec_heartbeat() {
                        break
                    }

                    if let Some(remaining) = Duration::from_millis(
                        timeout_secs.into())
                        .checked_sub(start.elapsed()) {
                        thread::sleep(remaining);
                    }
                }
            }
            // break -> init()
        }
    }

    fn init(&self) {
        let _ = self.daemon_event_tx.send(DaemonEvent::RpcConnecting);

        // 初始化上报操作
        loop {
            info!("client_login");
            {
                if let Err(e) = self.client.client_login() {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            info!("client_key_report");
            {
                if let Err(e) = self.client.client_key_report() {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            info!("binding device");
            {
                if let Err(e) = self.client.binding_device() {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            info!("search_user_team");
            {
                if let Err(e) = self.client.search_user_team() {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }
            break
        }
        let _ = self.daemon_event_tx.send(DaemonEvent::RpcConnected);
    }

    fn exec_heartbeat(&self) -> Result<()> {
        let timeout_secs: u32 = HEARTBEAT_FREQUENCY;
        info!("proxy_heart_beat");
        loop {
            let start = Instant::now();

            loop {
                if let Ok(_) = self.client.client_heartbeat() {
                    return Ok(());
                } else {
                    error!("Heart beat send failed.");
                }

                if Instant::now().duration_since(start) > Duration::from_secs(3) {
                    return Err(Error::RpcTimeout);
                }
                thread::sleep(Duration::from_millis(100));
            }

            if let Some(remaining) = Duration::from_secs(
                timeout_secs.into())
                .checked_sub(start.elapsed()) {
                thread::sleep(remaining);
            }
        }
    }

//    fn exec_online_proxy(&self) -> Result<()> {
//        info!("exec_online_proxy");
//        let timeout_secs = Duration::from_secs(3);
//        let start = Instant::now();
//        loop {
//            if let Ok(mut info) = self.info_arc.try_lock() {
//                if let Ok(_) = self.client.proxy_get_online_proxy(&mut info) {
//                    return Ok(());
//                } else {
//                    error!("proxy_get_online_proxy failed.");
//                }
//            }
//
//            if Instant::now().duration_since(start) > timeout_secs {
//                return Err(Error::RpcTimeout);
//            }
//            thread::sleep(Duration::from_millis(100));
//        }
//    }
}