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
    settings.common.username = user.user;
    settings.common.password = user.password;

    let (rpc_restart_tx, rpc_restart_rx) = mpsc::channel::<Response>();
    let _ = rpc_command_tx.send(RpcEvent::Client(RpcClientCmd::RestartRpcConnect));
    let response =
        if let Ok(res) = rpc_restart_rx.recv_timeout(Duration::from_secs(10)) {
            res
        }
        else {
            Response::exec_timeout()
        };
    let _ = Daemon::oneshot_send(tx, response, "");
}