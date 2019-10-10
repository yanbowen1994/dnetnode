use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use crate::daemon::{DaemonEvent, TunnelCommand};
use crate::traits::RpcTrait;
use crate::settings::get_settings;
use crate::tinc_manager::TincOperator;
use crate::rpc::rpc_cmd::{RpcCmd, RpcProxyCmd};

use super::web_server;
use super::RpcClient;
use std::sync::mpsc::Receiver;
use crate::rpc::proxy::report_host_status_change::report_host_status_change;
use crate::settings::default_settings::HEARTBEAT_FREQUENCY_SEC;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Connection with conductor timeout")]
    RpcTimeout,
}

pub struct RpcMonitor {
    client:                     RpcClient,
    daemon_event_tx:            mpsc::Sender<DaemonEvent>,
}

impl RpcTrait for RpcMonitor {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> mpsc::Sender<RpcCmd> {
        let (rpc_cmd_tx, rpc_cmd_rx) = mpsc::channel();
        let client = RpcClient::new();
        RpcMonitor {
            client,
            daemon_event_tx,
        }.start_monitor(rpc_cmd_rx);
        return rpc_cmd_tx;
    }
}

impl RpcMonitor {
    fn start_monitor(self, rpc_cmd_rx: Receiver<RpcCmd>) {
        let web_server_tx = self.daemon_event_tx.clone();
        thread::spawn(move || Self::cmd_handle(rpc_cmd_rx));
        thread::spawn(move ||
            web_server(Arc::new(Mutex::new(
                TincOperator::new())),
                       web_server_tx,
            )
        );
        thread::spawn(||self.run());
    }

    fn cmd_handle(rpc_cmd_rx: mpsc::Receiver<RpcCmd>) {
        while let Ok(rpc_cmd) = rpc_cmd_rx.recv() {
            match rpc_cmd {
                RpcCmd::Proxy(cmd) => {
                    match cmd {
                        RpcProxyCmd::HostStatusChange(host_status_change) => {
                            report_host_status_change(host_status_change);
                        }
                    }
                }
                _ => ()
            }
        }
    }

    fn run(self) {
        let timeout_secs: u32 = HEARTBEAT_FREQUENCY_SEC;
        loop {
            self.init();
            loop {
                let start = Instant::now();

                if let Err(_) = self.exec_heartbeat() {
                    break
                }

                if let Err(_) = self.exec_online_proxy() {
                    break
                }

                if let Some(remaining) = Duration::from_secs(
                    timeout_secs.into())
                    .checked_sub(start.elapsed()) {
                    thread::sleep(remaining);
                }
            }
            // break -> init()
        }
    }

    fn init(&self) {
        let _ = self.daemon_event_tx.send(DaemonEvent::RpcConnecting);
        let settings = get_settings();

        // 初始化上报操作
        loop {
            // RpcClient Login
            info!("proxy_login");
            {
                if let Err(e) = self.client.proxy_login(&settings) {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            // 注册proxy
            info!("proxy_register");
            {
                if let Err(e) = self.client.proxy_register() {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            // 初次上传heartbeat
            info!("proxy_heart_beat");
            {
                if let Err(e) = self.client.proxy_heart_beat() {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            info!("proxy_get_online_proxy");
            {
                if let Err(e) = self.client.proxy_get_online_proxy() {
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
        info!("proxy_heart_beat");
        let timeout_secs = Duration::from_secs(3);
        let start = Instant::now();
        loop {
            if let Ok(_) = self.client.proxy_heart_beat() {
                return Ok(());
            } else {
                error!("Heart beat send failed.");
            }

            if Instant::now().duration_since(start) > timeout_secs {
                return Err(Error::RpcTimeout);
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn exec_online_proxy(&self) -> Result<()> {
        info!("exec_online_proxy");
        let timeout_secs = Duration::from_secs(3);
        let start = Instant::now();
        loop {
            if let Ok(need_restart_tunnel) = self.client.proxy_get_online_proxy() {
                if need_restart_tunnel {
                    let _ = self.daemon_event_tx.send(DaemonEvent::DaemonInnerCmd(TunnelCommand::Reconnect));
                }
                return Ok(());
            } else {
                error!("proxy_get_online_proxy failed.");
            }

            if Instant::now().duration_since(start) > timeout_secs {
                return Err(Error::RpcTimeout);
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}