use std::sync::mpsc::Sender;
use dnet_types::response::Response;
use dnet_types::tinc_host_status_change::HostStatusChange;

pub enum RpcCmd {
    Client(RpcClientCmd),
    Proxy(RpcProxyCmd)
}

pub enum RpcClientCmd {
    StartHeartbeat,
    RestartRpcConnect,
    JoinTeam(String, Sender<Response>),
    ReportDeviceSelectProxy(Sender<Response>),
}

pub enum RpcProxyCmd {
    HostStatusChange(HostStatusChange),
}