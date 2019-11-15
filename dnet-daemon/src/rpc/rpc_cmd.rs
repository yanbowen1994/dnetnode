use std::sync::mpsc::Sender;
use dnet_types::response::Response;
use dnet_types::tinc_host_status_change::HostStatusChange;

#[derive(Debug)]
pub enum RpcEvent {
    Client(RpcClientCmd),
    Proxy(RpcProxyCmd),
    Executor(ExecutorEvent),
}

#[derive(Debug)]
pub enum RpcClientCmd {
    Stop(Sender<bool>),
    HeartbeatStart,
    RestartRpcConnect(Sender<Response>),
    JoinTeam(String, Sender<Response>),
    OutTeam(String, Sender<Response>),
    FreshTeam(Sender<Response>),
    ReportDeviceSelectProxy(Sender<Response>),
}

#[derive(Debug)]
pub enum RpcProxyCmd {
    HostStatusChange(HostStatusChange),
}

#[derive(Debug)]
pub enum ExecutorEvent {
    InitFinish,
    InitFailed(Response),
    NeedRestartTunnel,
}