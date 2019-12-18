use std::sync::mpsc;
use std::time::Duration;

use futures::sync::oneshot;

use dnet_types::user::User;
use dnet_types::response::Response;
use crate::rpc::rpc_cmd::{RpcEvent, RpcClientCmd};
use crate::settings::{get_mut_settings, get_settings};
use crate::daemon::Daemon;
use crate::info::get_info;
use super::handle_settings;

pub fn handle_login(ipc_tx: oneshot::Sender<Response>,
                    user: User,
                    rpc_command_tx: mpsc::Sender<RpcEvent>
) {
    if let Some(ipc_tx) = handle_settings::check_conductor_url(ipc_tx) {
        {
            let settings = get_mut_settings();
            settings.common.username = user.name;
            settings.common.password = user.password;
        }

        info!("handle_login send rpc cmd.");
        let response;
        let (rpc_restart_tx, rpc_restart_rx) = mpsc::channel::<Response>();
        if let Ok(_) = rpc_command_tx.send(
            RpcEvent::Client(RpcClientCmd::RestartRpcConnect(rpc_restart_tx))
        ) {
            response =
                if let Ok(mut res) = rpc_restart_rx.recv_timeout(
                    Duration::from_secs(
                        get_settings().common.http_timeout as u64 * 2
                    )) {
                    info!("handle_login {:?}", res);
                    let info = get_info().lock().unwrap();
                    let user = info.user.clone();
                    std::mem::drop(info);
                    let value = user.to_json();
                    res.data = Some(value);
                    res
                }
                else {
                    Response::exec_timeout()
                };
        }
        else {
            response = Response::internal_error();
        }

        let _ = Daemon::oneshot_send(ipc_tx, response, "");
    }
}