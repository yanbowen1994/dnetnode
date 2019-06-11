use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use crate::http_server_client::Client;
use crate::domain::Info;
use crate::daemon::DaemonEvent;
use crate::tinc_manager::TincOperator;

const HEARTBEAT_FREQUENCY: u32 = 20;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Connection with conductor timeout")]
    RpcTimeout,
}

pub struct RpcMonitor {
    client_arc:                 Arc<Mutex<Client>>,
    info_arc:                   Arc<Mutex<Info>>,
    daemon_event_tx:            mpsc::Sender<DaemonEvent>,
}

pub fn spawn(
    client_arc:                 Arc<Mutex<Client>>,
    info_arc:                   Arc<Mutex<Info>>,
    daemon_event_tx:            mpsc::Sender<DaemonEvent>,
) {
    RpcMonitor::new(client_arc, info_arc, daemon_event_tx).spawn();
}

impl RpcMonitor {
    pub fn new(
        client_arc:                 Arc<Mutex<Client>>,
        info_arc:                   Arc<Mutex<Info>>,
        daemon_event_tx:            mpsc::Sender<DaemonEvent>,
    ) -> Self {
        RpcMonitor {
            client_arc,
            info_arc,
            daemon_event_tx,
        }
    }

    pub fn spawn(self) {
        thread::spawn(||self.run());
    }

    fn run(self) {
        let timeout_secs: u32 = HEARTBEAT_FREQUENCY;
        loop {
            let start = Instant::now();
            if let Err(_) = self.exec_heartbeat() {
                self.daemon_event_tx.send(DaemonEvent::RpcFailed);
                return;
            }

            if let Err(_) = self.exec_online_proxy() {
                self.daemon_event_tx.send(DaemonEvent::RpcFailed);
                return;
            }

            if let Some(remaining) =
            Duration::from_secs(timeout_secs.into()).checked_sub(start.elapsed()) {
                thread::sleep(remaining);
            }
        }
    }

    fn exec_heartbeat(&self) -> Result<()> {
        info!("proxy_heart_beat");
        let timeout_secs = Duration::from_secs(3);
        let start = Instant::now();
        loop {
            if let Ok(client) = self.client_arc.try_lock() {
                if let Ok(info) = self.info_arc.try_lock() {
                    if let Ok(_) = client.proxy_heart_beat(&info) {
                        return Ok(());
                    } else {
                        error!("Heart beat send failed.");
                    }
                }
            };

            if Instant::now().duration_since(start) > timeout_secs {
                return Err(Error::RpcTimeout);
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn exec_online_proxy(&self) -> Result<()>{
        info!("exec_online_proxy");
        let timeout_secs = Duration::from_secs(3);
        let start = Instant::now();
        loop {
            if let Ok(client) = self.client_arc.try_lock() {
                if let Ok(mut info) = self.info_arc.try_lock() {
                    if let Ok(_) = client.proxy_get_online_proxy(&mut info) {
                        return Ok(());
                    } else {
                        error!("proxy_get_online_proxy failed.");
                    }
                }
            }
            if Instant::now().duration_since(start) > timeout_secs {
                return Err(Error::RpcTimeout);
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}