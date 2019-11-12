use std::sync::{mpsc};

use futures::sync::oneshot;

use crate::rpc::rpc_cmd::{RpcEvent, RpcClientCmd};
use dnet_types::user::User;
use crate::settings::get_mut_settings;
use dnet_types::response::Response;
use std::time::Duration;
use crate::daemon::Daemon;

pub fn handle_login(tx: oneshot::Sender<Response>, user: User, rpc_command_tx: mpsc::Sender<RpcEvent>) {
    let settings = get_mut_settings();
    settings.common.username = user.name;
    settings.common.password = user.password;

    info!("handle_login send rpc cmd.");
    let response;
    let (rpc_restart_tx, rpc_restart_rx) = mpsc::channel::<Response>();
    if let Ok(_) = rpc_command_tx.send(
        RpcEvent::Client(RpcClientCmd::RestartRpcConnect(rpc_restart_tx))
    ) {
        response =
            if let Ok(res) = rpc_restart_rx.recv_timeout(Duration::from_secs(10)) {
                info!("handle_login {:?}", res);
                res
            }
            else {
                Response::exec_timeout()
            };
    }
    else {
        response = Response::internal_error();
    }

    let _ = Daemon::oneshot_send(tx, response, "");
}