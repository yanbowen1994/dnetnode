use crate::states::{TunnelState, RpcState};
use serde::{Serialize, Deserialize};

/// An event sent out from the daemon to frontends.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DaemonBroadcast<T: Serialize> {
    /// The daemon transitioned into a new state.
    TunnelState(TunnelState),

    /// The daemon transitioned into a new state.
    RpcState(RpcState),

    /// The daemon settings changed.
    Settings(T),
}