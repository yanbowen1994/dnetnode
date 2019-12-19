use crate::status::{TunnelState, RpcState};
use crate::settings::Settings;

/// An event sent out from the daemon to frontends.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DaemonBroadcast {
    /// The daemon transitioned into a new Status.
    TunnelState(TunnelState),

    /// The daemon transitioned into a new Status.
    RpcState(RpcState),

    /// The daemon settings changed.
    Settings(Settings),
}