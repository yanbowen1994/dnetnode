use std::sync::mpsc;
use std::time::Duration;

use futures::sync::oneshot;

use dnet_types::response::Response;
use dnet_types::status::RpcState;
use dnet_types::settings::RunMode;
use crate::settings::get_settings;
use crate::daemon::Daemon;
use crate::rpc::rpc_cmd::{RpcEvent, RpcClientCmd};
use crate::info::{get_mut_info, get_info};

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
) -> Option<oneshot::Sender<Response>> {
    if get_info().lock().unwrap().status.rpc == RpcState::Connected {
        return Some(ipc_tx)
    }
    else {
        let response = Response::not_login();
        let _ = Daemon::oneshot_send(ipc_tx, response, "");
        return None;
    }
}

pub fn send_rpc_group_fresh(
    rpc_command_tx: mpsc::Sender<RpcEvent>
) -> Response {
    let (res_tx, res_rx) = mpsc::channel();
    let _ = rpc_command_tx.send(RpcEvent::Client(RpcClientCmd::FreshTeam(res_tx)));
    if let Ok(res) = res_rx.recv_timeout(Duration::from_secs(
        get_settings().common.http_timeout as u64
    )) {
        res
    }
    else {
        Response::exec_timeout()
    }
}

pub fn daemon_event_handle_fresh_running_from_all() {
    let mut info = get_mut_info().lock().unwrap();
    info.fresh_running_from_all();
    info!("fresh_running_from_all running teams {:?}", info.teams.running_teams);
}