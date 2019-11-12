use std::sync::mpsc::Sender;
use dnet_types::response::Response;
use dnet_types::tinc_host_status_change::HostStatusChange;

use super::error::Error;
use super::client::SubError;

pub enum RpcEvent {
    Client(RpcClientCmd),
    Proxy(RpcProxyCmd),
    Executor(ExecutorEvent),
}

pub enum RpcClientCmd {
    HeartbeatStart,
    Stop,
    RestartRpcConnect(Sender<Response>),
    JoinTeam(String, Sender<Response>),
    ReportDeviceSelectProxy(Sender<Response>),
}

pub enum RpcProxyCmd {
    HostStatusChange(HostStatusChange),
}

pub enum ExecutorEvent {
    InitFinish,
    InitFailed(SubError),
    NeedRestartTunnel,
}