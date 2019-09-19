use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use crate::daemon::DaemonEvent;
use crate::traits::RpcTrait;
use crate::settings::get_settings;
use crate::info::Info;
use crate::tinc_manager::TincOperator;

use super::RpcClient;

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
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>)
        -> Self {
        let client = RpcClient::new();
        return RpcMonitor {
            client,
            daemon_event_tx,
        };
    }

    fn start_monitor(self) {
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

        // 初始化上报操作
        loop {
            // RpcClient Login
            info!("client_login");
            {
                if let Err(e) = self.client.client_login() {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            // binding device
            info!("binding device");
            {
                if let Err(e) = self.client.binding_device() {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            info!("search_team_by_mac");
            {
                if let Err(e) = self.client.search_team_by_mac() {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

//            // 初次上传heartbeat
//            info!("proxy_heart_beat");
//            {
//                if let Err(e) = self.client.proxy_heart_beat(&mut info) {
//                    error!("{:?}\n{}", e, e);
//                    thread::sleep(std::time::Duration::from_secs(1));
//                    continue
//                }
//            }
//            break
        }
        let _ = self.daemon_event_tx.send(DaemonEvent::RpcConnected);
    }

    fn exec_heartbeat(&self) -> Result<()> {
        info!("proxy_heart_beat");
        let timeout_secs = Duration::from_secs(3);
        let start = Instant::now();
        loop {
            if let Ok(_) = self.client.client_heartbeat() {
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