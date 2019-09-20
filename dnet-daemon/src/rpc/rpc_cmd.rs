pub enum RpcCmd {
    Client(RpcClientCmd),
    Proxy(RpcProxyCmd)
}

pub enum RpcClientCmd {
    StartHeartbeat,
    RestartRpcConnect,
    JoinTeam(String),
}

pub enum RpcProxyCmd {

}