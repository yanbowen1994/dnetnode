use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use crate::daemon::DaemonEvent;
use crate::traits::RpcTrait;
use crate::tinc_manager::TincOperator;
use crate::rpc::rpc_cmd::{RpcEvent, RpcProxyCmd};

use super::web_server;
use super::RpcClient;
use std::sync::mpsc::Receiver;
use crate::settings::default_settings::HEARTBEAT_FREQUENCY_SEC;
use crate::settings::get_settings;
use dnet_types::settings::RunMode;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Connection with conductor timeout")]
    RpcTimeout,
    #[error(display = "InitWebServer")]
    InitWebServer,
    #[error(display = "InitRpcMonitor")]
    InitRpcMonitor,
}

pub struct RpcMonitor {
    client:                     RpcClient,
    daemon_event_tx:            mpsc::Sender<DaemonEvent>,
}

impl RpcTrait for RpcMonitor {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> Option<mpsc::Sender<RpcEvent>> {
        let (rpc_tx, rpc_rx) = mpsc::channel();
        let client = RpcClient::new();
        RpcMonitor {
            client,
            daemon_event_tx,
        }.start_monitor(rpc_rx)
            .ok()?;
        return Some(rpc_tx);
    }
}

impl RpcMonitor {
    fn start_monitor(self, rpc_rx: Receiver<RpcEvent>) -> Result<()> {
        let web_server_tx = self.daemon_event_tx.clone();

        thread::Builder::new()
            .name("web_server".to_string())
            .spawn(||
                web_server(Arc::new(Mutex::new(
                    TincOperator::new())),
                           web_server_tx,
                )
            )
            .map_err(|_|Error::InitWebServer)?;


        thread::Builder::new()
            .name("rpc_cmd_handle".to_string())
            .spawn(||cmd_handle(rpc_rx))
            .map_err(|_|Error::InitRpcMonitor)?;

        thread::Builder::new()
            .name("RpcMonitor".to_string())
            .spawn(||self.run())
            .map_err(|_|Error::InitRpcMonitor)?;

        Ok(())
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
        let run_mode = get_settings().common.mode.clone();

        let _ = self.daemon_event_tx.send(DaemonEvent::RpcConnecting);

        // 初始化上报操作
        loop {
            // RpcClient Login
            info!("proxy_login");
            {
                if let Err(e) = self.client.proxy_login() {
                    error!("proxy_login {:?} {}", e, e.get_http_error_msg());
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            // 注册proxy
            info!("proxy_add");
            {
                if let Err(e) = self.client.proxy_add() {
                    error!("proxy_add {:?} {}", e, e.get_http_error_msg());
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            // 初次上传heartbeat
            info!("proxy_heart_beat");
            {
                if let Err(e) = self.client.proxy_heartbeat() {
                    error!("proxy_heart_beat {:?} {}", e, e.get_http_error_msg());
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            if run_mode == RunMode::Proxy {
                info!("all_device_pubkey");
                {
                    if let Err(e) = self.client.all_device_pubkey() {
                        error!("all_device_pubkey {:?} {}", e, e.get_http_error_msg());
                        thread::sleep(std::time::Duration::from_secs(1));
                        continue
                    }
                }
            }

            info!("proxy_get_online_proxy");
            {
                match self.client.proxy_get_online_proxy() {
                    Ok(connect_to_vec) => {
                        self.client.init_connect_to(connect_to_vec);
                    }
                    Err(e) => {
                        error!("proxy_get_online_proxy {:?} {}", e, e.get_http_error_msg());
                        thread::sleep(std::time::Duration::from_secs(1));
                        continue
                    }
                }
            }
            break
        }
        let _ = self.daemon_event_tx.send(DaemonEvent::RpcConnected);
    }

    #[allow(unused_assignments)]
    fn exec_heartbeat(&self) -> Result<()> {
        info!("proxy_heart_beat");
        let timeout_secs = Duration::from_secs(20);
        let start = Instant::now();

        loop {
            if let Ok(_) = self.client.proxy_heartbeat() {
                return Ok(());
            } else {
                error!("Heart beat send failed.");
            }

            if Instant::now().duration_since(start) > timeout_secs {
                return Err(Error::RpcTimeout);
            }
            thread::sleep(Duration::from_millis(1000));
        }
    }

    fn exec_online_proxy(&self) -> Result<()> {
        trace!("exec_online_proxy");
        let timeout_secs = Duration::from_secs(20);
        let start = Instant::now();
        loop {
            if let Ok(connect_to) = self.client.proxy_get_online_proxy() {
                self.client.add_connect_to_host(connect_to);
                return Ok(());
            } else {
                error!("proxy_get_online_proxy failed.");
            }

            if Instant::now().duration_since(start) > timeout_secs {
                return Err(Error::RpcTimeout);
            }
            thread::sleep(Duration::from_millis(1000));
        }
    }
}

fn cmd_handle(rpc_rx: Receiver<RpcEvent>) {
    while let Ok(rpc_cmd) = rpc_rx.recv() {
        info!("rpc event {:?}", rpc_cmd);
        match rpc_cmd {
            RpcEvent::Proxy(cmd) => {
                match cmd {
                    RpcProxyCmd::HostStatusChange(host_status_change) => {
                        let _ = thread::Builder::new()
                            .name("update_tinc_status".to_string())
                            .spawn( move||
                                if let Err(e) = RpcClient::new()
                                    .center_update_tinc_status(host_status_change) {
                                    error!("{:?}", e.to_response());
                                }
                        );
                    }
                }
            },
            RpcEvent::TunnelConnected => {
                if get_settings().common.mode == RunMode::Center {
                    if let Err(e) = RpcClient::new()
                        .center_get_team_info() {
                        error!("center_get_team_info {:?}", e.to_response());
                    }
                }
            },
            _ => ()
        }
    }
}