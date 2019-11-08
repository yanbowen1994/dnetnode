use std::sync::mpsc::Sender;
use dnet_types::response::Response;
use dnet_types::tinc_host_status_change::HostStatusChange;

pub enum RpcEvent {
    Client(RpcClientCmd),
    Proxy(RpcProxyCmd),
    Executor(ExecutorEvent),
}

pub enum RpcClientCmd {
    HeartbeatStart,
    Stop,
    RestartRpcConnect(Sender<bool>),
    JoinTeam(String, Sender<Response>),
    ReportDeviceSelectProxy(Sender<Response>),
}

pub enum RpcProxyCmd {
    HostStatusChange(HostStatusChange),
}

pub enum ExecutorEvent {
    NeedRestartTunnel,
}