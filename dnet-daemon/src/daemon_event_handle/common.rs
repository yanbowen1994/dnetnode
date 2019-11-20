use futures::sync::oneshot;

use dnet_types::response::Response;
use dnet_types::states::{RpcState, State};
use dnet_types::settings::RunMode;
use crate::settings::get_settings;
use crate::daemon::Daemon;

pub fn is_not_proxy(ipc_tx: oneshot::Sender<Response>) -> Option<oneshot::Sender<Response>> {
    let run_mode = get_settings().common.mode.clone();
    if run_mode == RunMode::Proxy || run_mode == RunMode::Center {
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
