use std::sync::mpsc;
use std::time::Duration;

use futures::sync::oneshot;

use dnet_types::response::Response;
use dnet_types::states::{TunnelState, State};
use crate::rpc::rpc_cmd::{RpcEvent, RpcClientCmd};
use crate::daemon::{Daemon, TunnelCommand};
use crate::info::{get_info, get_mut_info};
use super::tunnel::send_tunnel_disconnect;
use super::common::is_not_proxy;
use crate::daemon_event_handle::common::is_rpc_connected;

pub fn group_out(
    ipc_tx:                 oneshot::Sender<Response>,
    team_id:                String,
    status:                 State,
    rpc_command_tx:         mpsc::Sender<RpcEvent>,
    tunnel_command_tx:      mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>,
) {
    info!("is_not_proxy");
    let _ = is_not_proxy(ipc_tx)
        .and_then(|ipc_tx|{
            info!("is_rpc_connected");
            is_rpc_connected(ipc_tx, &status)
        })
        .and_then(|ipc_tx|{
            info!("is_joined");
            if is_joined(&team_id) {
                info!("send_rpc_out_group");
                send_rpc_out_group(&team_id, ipc_tx, rpc_command_tx.clone())
            }
            else {
                Some(ipc_tx)
            }
        })
        .and_then(|ipc_tx| {
            info!("del_start_team");
            del_local_team(&team_id);
            info!("need_tunnel_disconnect");
            if need_tunnel_disconnect(&status) {
                info!("handle_tunnel_disconnect");
                handle_tunnel_disconnect(ipc_tx, tunnel_command_tx)
            }
            else {
                Some(ipc_tx)
            }
        })
        .and_then(|ipc_tx| {
            info!("handle_rpc_stop_heartbeat");
            handle_rpc_stop_heartbeat(ipc_tx, rpc_command_tx)
        })
        .and_then(|ipc_tx| {
            info!("success");
            let response = Response::success();
            let _ = Daemon::oneshot_send(ipc_tx, response, "");
            Some(())
        });
}

fn is_joined(team_id: &str) -> bool {
    let info = get_info().lock().unwrap();
    let is_joined = info.teams.all_teams.contains_key(team_id);

    if is_joined {
        return true;
    }
    else {
        return false;
    }
}

fn send_rpc_out_group(
    team_id: &str,
    ipc_tx: oneshot::Sender<Response>,
    rpc_command_tx: mpsc::Sender<RpcEvent>,
) -> Option<oneshot::Sender<Response>> {
    let (res_tx, res_rx) = mpsc::channel();
    let _ = rpc_command_tx.send(
        RpcEvent::Client(RpcClientCmd::OutTeam(team_id.to_owned(), res_tx)));
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

fn del_local_team(team_id: &str) {
    let mut info = get_mut_info().lock().unwrap();
    info.teams.all_teams.remove(team_id);
    info.teams.del_start_team(team_id);
}

fn need_tunnel_disconnect(status: &State) -> bool {
    let info = get_info().lock().unwrap();
    if info.teams.running_teams.len() == 0 {
        if status.tunnel == TunnelState::Connected
            || status.tunnel == TunnelState::Connecting {
            return true;
        }
        else {
            return false;
        }
    }
    else {
        return false;
    }
}

fn handle_tunnel_disconnect(
    ipc_tx:             oneshot::Sender<Response>,
    tunnel_command_tx:  mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>,
) -> Option<oneshot::Sender<Response>> {
    let response = send_tunnel_disconnect(tunnel_command_tx);
    if response.code == 200 {
        return Some(ipc_tx);
    }
    else {
        let _ = Daemon::oneshot_send(ipc_tx, response, "");
        return None;
    }
}

fn handle_rpc_stop_heartbeat(
    ipc_tx:             oneshot::Sender<Response>,
    rpc_command_tx:     mpsc::Sender<RpcEvent>,
) -> Option<oneshot::Sender<Response>> {
    let (res_tx, res_rx) = mpsc::channel();
    let _ = rpc_command_tx.send(RpcEvent::Client(RpcClientCmd::Stop(res_tx)));
    if let Ok(_) = res_rx.recv_timeout(Duration::from_secs(3)) {
        return Some(ipc_tx);
    }
    else {
        let response = Response::exec_timeout();
        let _ = Daemon::oneshot_send(ipc_tx, response, "");
        return None;
    }
}