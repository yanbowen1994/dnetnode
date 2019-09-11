#[derive(Clone, Debug)]
pub enum ManagementCommand {
    // Tunnel
    TunnelConnect,
    TunnelDisConnect,
    TunnelStatus,
    // Rpc
    RpcStatus,
    // Group
    GroupInfo,
}