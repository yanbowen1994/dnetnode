use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use crate::daemon::DaemonEvent;
use crate::traits::RpcTrait;
use crate::settings::get_settings;
use crate::info::{Info, get_info};
use crate::tinc_manager::TincOperator;

use super::RpcClient;
use crate::rpc::proxy::web_server;

const HEARTBEAT_FREQUENCY: u32 = 20;

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
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>)-> Self {
        let client = RpcClient::new();
        return RpcMonitor {
            client,
            daemon_event_tx,
        };
    }

    fn start_monitor(self) {
        let web_server_tx = self.daemon_event_tx.clone();

        let info_arc_clone = get_info();
        thread::spawn(move ||
            web_server(Arc::new(Mutex::new(
                           TincOperator::new())),
                       web_server_tx,
            )
        );
        thread::spawn(||self.run());
    }
}

impl RpcMonitor {
    fn run(self) {
        let timeout_secs: u32 = HEARTBEAT_FREQUENCY;
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
            if let Ok(_) = self.client.proxy_get_online_proxy() {
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