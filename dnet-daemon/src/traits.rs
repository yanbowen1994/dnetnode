use std::sync::{Arc, Mutex, mpsc};
use crate::daemon::{DaemonEvent, TunnelCommand};
use crate::rpc::rpc_cmd::RpcCmd;

pub enum RpcRequest {
    Status,
}

pub trait RpcTrait {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> mpsc::Sender<RpcCmd>;
}

pub trait InfoTrait {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> Self;
    fn create_uid(&mut self);
}

pub trait TunnelTrait
    where Self: std::marker::Sized
{
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> (Self, mpsc::Sender<TunnelCommand>);
    fn start_monitor(self);
}