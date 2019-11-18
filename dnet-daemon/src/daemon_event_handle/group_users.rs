use std::sync::mpsc;
use std::time::Duration;

use futures::sync::oneshot;

use dnet_types::response::Response;
use crate::rpc::rpc_cmd::{RpcEvent, RpcClientCmd};
use crate::daemon::Daemon;

pub fn handle_group_users(
    ipc_tx:                 oneshot::Sender<Response>,
    rpc_command_tx:         mpsc::Sender<RpcEvent>,
    team_id:                String,
) {
    info!("handle_group_users");
    let res = send_rpc_team_users(rpc_command_tx, team_id);
    let _ = Daemon::oneshot_send(ipc_tx, res, "");
}

fn send_rpc_team_users(
    rpc_command_tx:     mpsc::Sender<RpcEvent>,
    team_id:            String,
) -> Response {
    let (res_tx, res_rx) = mpsc::channel();
    let _ = rpc_command_tx.send(RpcEvent::Client(RpcClientCmd::TeamUsers(team_id, res_tx)));
    if let Ok(res) = res_rx.recv_timeout(Duration::from_secs(5)) {
        res
    }
    else {
        Response::exec_timeout()
    }
}