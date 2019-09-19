use std::sync::{Arc, Mutex, mpsc};

use crate::daemon::DaemonEvent;
use crate::traits::RpcTrait;
use crate::info::Info;

use super::client;
use super::proxy;

pub struct RpcMonitor<RpcInner> {
    inner: RpcInner,
}

impl<RpcInner> RpcTrait for RpcMonitor<RpcInner>
    where RpcInner: RpcTrait,
{
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> Self {
        return RpcMonitor {
            inner: RpcInner::new(daemon_event_tx)
        };
    }

    fn start_monitor(self) {
        self.inner.start_monitor()
    }
}