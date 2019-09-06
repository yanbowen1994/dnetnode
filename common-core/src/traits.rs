use std::sync::{Arc, Mutex, mpsc};
use crate::daemon::DaemonEvent;

pub enum RpcRequest {
    Status,
}

pub trait RpcTrait<Info>
    where Info: InfoTrait
{
    fn new(info_arc: Arc<Mutex<Info>>, daemon_event_tx: mpsc::Sender<DaemonEvent>) -> Self;
    fn start_monitor(self);
}

pub trait InfoTrait {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> Self;
    fn create_uid(&mut self);
}

pub trait TunnelTrait {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> Self;
    fn start_monitor(self);
}