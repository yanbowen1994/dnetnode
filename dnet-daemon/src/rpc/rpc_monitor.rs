use std::sync::{Arc, Mutex, mpsc};

use crate::daemon::DaemonEvent;
use crate::traits::RpcTrait;
use crate::info::Info;

use super::client;
use super::proxy;

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