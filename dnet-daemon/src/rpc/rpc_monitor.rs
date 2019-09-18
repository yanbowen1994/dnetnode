use std::sync::{Arc, Mutex, mpsc};
//use std::thread;
//use std::time::{Duration, Instant};
//
use crate::daemon::DaemonEvent;
use crate::traits::RpcTrait;
use crate::settings::get_settings;
use crate::info::Info;
//use crate::tinc_manager::TincOperator;

use super::client;
use super::proxy;
use dnet_types::settings::RunMode;

//const HEARTBEAT_FREQUENCY: u32 = 20;
//
//pub type Result<T> = std::result::Result<T, Error>;
//
//#[derive(err_derive::Error, Debug)]
//pub enum Error {
//    #[error(display = "Connection with conductor timeout")]
//    RpcTimeout,
//}

enum RpcInner {
    Client(client::RpcMonitor),
    Proxy(proxy::RpcMonitor),
}

pub struct RpcMonitor<RpcInner> {
    inner: RpcInner,
}

impl<RpcInner> RpcTrait<Info> for RpcMonitor<RpcInner>
    where RpcInner: RpcTrait<Info>,
{
    fn new(info_arc: Arc<Mutex<Info>>,
           daemon_event_tx: mpsc::Sender<DaemonEvent>)
        -> Self
    {
        return RpcMonitor {
            inner: RpcInner::new(info_arc, daemon_event_tx)
        };
    }

    fn start_monitor(self) {
        self.inner.start_monitor()
    }
}