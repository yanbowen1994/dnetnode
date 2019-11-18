use std::sync::mpsc;
use std::time::Duration;

use futures::sync::oneshot;

use dnet_types::user::User;
use dnet_types::response::Response;
use dnet_types::states::{TunnelState, RpcState, State};
use dnet_types::settings::RunMode;
use crate::rpc::rpc_cmd::{RpcEvent, RpcClientCmd};
use crate::settings::{get_mut_settings, get_settings};
use crate::daemon::{Daemon, TunnelCommand, DaemonEvent};
use crate::info::{get_info, get_mut_info};
use super::tunnel::send_tunnel_connect;
use super::handle_settings;

pub fn is_not_proxy(ipc_tx: oneshot::Sender<Response>) -> Option<oneshot::Sender<Response>> {
    let run_mode = get_settings().common.mode.clone();
    if run_mode == RunMode::Proxy {
        let response = Response::internal_error()
            .set_msg("Invalid command in proxy mode".to_owned());
        let _ = Daemon::oneshot_send(ipc_tx, response, "");
        return None;
    }
    else {
        return Some(ipc_tx);
    }
}

pub fn is_rpc_connected(
    ipc_tx:   oneshot::Sender<Response>,
    status:   &State,
) -> Option<oneshot::Sender<Response>> {
    if status.rpc == RpcState::Connected {
        return Some(ipc_tx)
    }
    else {
        let response = Response::not_login();
        let _ = Daemon::oneshot_send(ipc_tx, response, "");
        return None;
    }
}
