use std::sync::mpsc;
use std::time::Duration;

use futures::sync::oneshot;

use dnet_types::response::Response;
use dnet_types::states::{State, TunnelState};
use crate::rpc::rpc_cmd::{RpcEvent, RpcClientCmd};
use crate::daemon::{Daemon, TunnelCommand};
use crate::info::get_mut_info;
use super::tunnel::send_tunnel_connect;
use super::handle_settings;
use super::common::is_not_proxy;
use crate::daemon_event_handle::common::{is_rpc_connected, send_rpc_group_fresh};

pub fn group_join(
    ipc_tx:                 oneshot::Sender<Response>,
    team_id:                String,
    status:                 State,
    rpc_command_tx:         mpsc::Sender<RpcEvent>,
    tunnel_command_tx:      mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>,
) {
    info!("is_not_proxy");
    let _ = is_not_proxy(ipc_tx)
        .and_then(|ipc_tx| {
            info!("check_conductor_url");
            handle_settings::check_conductor_url(ipc_tx)
        })
        .and_then(|ipc_tx|{
            info!("is_rpc_connected");
            is_rpc_connected(ipc_tx, &status)
        })
        .and_then(|ipc_tx|{
            info!("send_rpc_join_group");
            send_rpc_join_group(&team_id, ipc_tx, rpc_command_tx.clone())
        })
        .and_then(|ipc_tx| {
            info!("add_start_team");
            add_start_team(&team_id);
            info!("need_tunnel_connect");
            if need_tunnel_connect(&status) {
                info!("handle_connect_select_proxy");
                handle_connect_select_proxy(ipc_tx, rpc_command_tx.clone())
                    .and_then(|ipc_tx| {
                        info!("handle_tunnel_connect");
                        handle_tunnel_connect(ipc_tx, tunnel_command_tx)
                    })
            }
            else {
                Some(ipc_tx)
            }
        })
        .and_then(|ipc_tx| {
            let response = send_rpc_group_fresh(rpc_command_tx);
            if response.code == 200{
                Some(ipc_tx)
            }
            else {
                let _ = Daemon::oneshot_send(ipc_tx, response, "");
                None
            }
        })
        .and_then(|ipc_tx| {
            let response = Response::success();
            let _ = Daemon::oneshot_send(ipc_tx, response, "");
            info!("success");
            Some(())
        });
}

fn send_rpc_join_group(
    team_id: &str,
    ipc_tx: oneshot::Sender<Response>,
    rpc_command_tx: mpsc::Sender<RpcEvent>
) -> Option<oneshot::Sender<Response>> {
    let (res_tx, res_rx) = mpsc::channel();
    let _ = rpc_command_tx.send(
        RpcEvent::Client(RpcClientCmd::JoinTeam(team_id.to_owned(), res_tx)));
    let response = match res_rx.recv_timeout(Duration::from_secs(3)) {
        Ok(res) => res,
        Err(_) => Response::exec_timeout(),
    };
    if response.code != 200 {
        let _ = Daemon::oneshot_send(ipc_tx, response, "");
        return None;
    }
    else {
        return Some(ipc_tx);
    }
}

fn add_start_team(team_id: &str) {
    let mut info = get_mut_info().lock().unwrap();
    info.teams.add_start_team(team_id);
}

fn need_tunnel_connect(status: &State) -> bool {
    if status.tunnel == TunnelState::Disconnected
        || status.tunnel == TunnelState::Disconnecting {
        true
    }
    else {
        false
    }
}

fn handle_connect_select_proxy(
    ipc_tx: oneshot::Sender<Response>,
    rpc_command_tx: mpsc::Sender<RpcEvent>,
) -> Option<oneshot::Sender<Response>> {
    let (rpc_response_tx, rpc_response_rx) = mpsc::channel();
    let _ = rpc_command_tx.send(
        RpcEvent::Client(RpcClientCmd::ReportDeviceSelectProxy(rpc_response_tx)));

    if let Ok(response) = rpc_response_rx.recv() {
        if response.code == 200 {
            return Some(ipc_tx);
        }
        else {
            let _ = Daemon::oneshot_send(ipc_tx, response, "");
            return None;
        }
    }
    else {
        let response = Response::internal_error().set_msg("Exec failed.".to_owned());
        let _ = Daemon::oneshot_send(ipc_tx, response, "");
        return None;
    }

}

fn handle_tunnel_connect(
    ipc_tx:             oneshot::Sender<Response>,
    tunnel_command_tx:  mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>,
) -> Option<oneshot::Sender<Response>> {
    let response = send_tunnel_connect(tunnel_command_tx);
    if response.code == 200 {
        return Some(ipc_tx);
    }
    else {
        let _ = Daemon::oneshot_send(ipc_tx, response, "");
        return None;
    }
}