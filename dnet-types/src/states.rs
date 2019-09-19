#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub rpc:        RpcState,
    pub tunnel:     TunnelState,
    pub daemon:     DaemonExecutionState,
}

impl State {
    pub fn new() -> Self {
        State {
            rpc:    RpcState::Connecting,
            tunnel: TunnelState::Disconnected,
            daemon: DaemonExecutionState::Running,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RpcState {
    Connecting,
    Connected,
    ReConnecting,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TunnelState {
    Connecting,
    Connected,
    Disconnecting,
    Disconnected,
    TunnelInitFailed(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DaemonExecutionState {
    Running,
    Finished,
}
