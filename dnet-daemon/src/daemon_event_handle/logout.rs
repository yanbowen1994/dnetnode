use std::sync::mpsc;
use std::time::Duration;

use futures::sync::oneshot;

use dnet_types::response::Response;
use dnet_types::status::RpcState;
use sandbox::route::keep_route;

use crate::rpc::rpc_cmd::{RpcEvent, RpcClientCmd};
use crate::settings::{get_mut_settings, get_settings};
use crate::daemon::{Daemon, TunnelCommand};
use crate::info::{get_mut_info, UserInfo, TincInfo};
use crate::daemon_event_handle::tunnel::send_tunnel_disconnect;
use crate::settings::default_settings::TINC_INTERFACE;

pub fn handle_logout(
    ipc_tx:             oneshot::Sender<Response>,
    rpc_command_tx:     mpsc::Sender<RpcEvent>,
    tunnel_command_tx:  mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>,
) {
    let _ = need_logout(ipc_tx)
        .and_then(|ipc_tx| {
            info!("send_rpc_disconnect");
            send_rpc_disconnect(ipc_tx, rpc_command_tx)})
        .and_then(|ipc_tx| {
            let response = send_tunnel_disconnect(tunnel_command_tx);
            get_mut_info().lock().unwrap().status.rpc = RpcState::Disconnected;
            clean_all_running_teams();
            clean_route_table();
            let _ = Daemon::oneshot_send(ipc_tx, response, "");
            Some(())
        });
}

fn need_logout(
    ipc_tx:             oneshot::Sender<Response>,
) -> Option<oneshot::Sender<Response>> {
    let settings = get_mut_settings();
    if settings.common.username.is_empty(){
        let response = Response::not_login();
        let _ = Daemon::oneshot_send(ipc_tx, response, "");
        None
    }
    else {
        info!("clean_settings_user.");
        clean_settings_user();
        info!("clean_info_user.");
        clean_info_user();
        Some(ipc_tx)
    }
}

fn clean_settings_user() {
    let settings = get_mut_settings();
    settings.common.username = "".to_owned();
    settings.common.password = "".to_owned();
}

fn clean_info_user() {
    let mut tinc_info = TincInfo::new();
    tinc_info.load_local();
    let mut info = get_mut_info().lock().unwrap();
    info.user = UserInfo::new();
    info.node.token = "".to_owned();
    info.tinc_info = tinc_info;
}

fn send_rpc_disconnect(
    ipc_tx:             oneshot::Sender<Response>,
    rpc_command_tx:     mpsc::Sender<RpcEvent>,
) -> Option<oneshot::Sender<Response>> {
    let (rpc_stop_tx, rpc_stop_rx) =
        mpsc::channel::<bool>();
    if let Ok(_) = rpc_command_tx.send(
        RpcEvent::Client(RpcClientCmd::Stop(rpc_stop_tx))
    ) {
        if let Ok(_) = rpc_stop_rx.recv_timeout(
            Duration::from_secs(
                get_settings().common.http_timeout as u64
            )) {
            return Some(ipc_tx);
        }
        else {
            let response = Response::exec_timeout();
            let _ = Daemon::oneshot_send(ipc_tx, response, "");
            return None;
        };
    }
    else {
        let response = Response::internal_error();
        let _ = Daemon::oneshot_send(ipc_tx, response, "");
        return None;
    }
}

fn clean_all_running_teams() {
    let mut info = get_mut_info().lock().unwrap();
    info.teams.running_teams = vec![];
    std::mem::drop(info);
}

fn clean_route_table() {
    keep_route(None, vec![], TINC_INTERFACE.to_string());
}