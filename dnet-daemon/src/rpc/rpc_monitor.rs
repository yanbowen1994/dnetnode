use std::sync::mpsc;

use crate::daemon::DaemonEvent;
use crate::traits::RpcTrait;

use super::rpc_cmd::RpcEvent;

pub struct RpcMonitor;

impl RpcMonitor {
    pub fn new<RpcInner>(daemon_event_tx: mpsc::Sender<DaemonEvent>)
        -> Option<mpsc::Sender<RpcEvent>>
        where RpcInner: RpcTrait,
    {
        RpcInner::new(daemon_event_tx)
    }
}