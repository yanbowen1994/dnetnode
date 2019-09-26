use std::sync::mpsc::Sender;
use dnet_types::response::Response;

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

}