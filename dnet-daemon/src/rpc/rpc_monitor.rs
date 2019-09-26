use std::sync::mpsc;

use crate::daemon::DaemonEvent;
use crate::traits::RpcTrait;

use super::rpc_cmd::RpcCmd;

pub struct RpcMonitor;

impl RpcMonitor {
    pub fn new<RpcInner>(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> mpsc::Sender<RpcCmd>
        where RpcInner: RpcTrait,
    {
        let rpc_cmd_tx = RpcInner::new(daemon_event_tx);
        return rpc_cmd_tx;
    }
}