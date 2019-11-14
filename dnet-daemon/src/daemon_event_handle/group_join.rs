use std::sync::{mpsc};

use futures::sync::oneshot;

use crate::rpc::rpc_cmd::{RpcEvent, RpcClientCmd};
use dnet_types::user::User;
use crate::settings::{get_mut_settings, get_settings};
use dnet_types::response::Response;
use std::time::Duration;
use crate::daemon::{Daemon, TunnelCommand, DaemonEvent};
use crate::info::{get_info, get_mut_info};
use dnet_types::states::{TunnelState, RpcState, State};
use dnet_types::settings::RunMode;
use super::tunnel::send_tunnel_connect;

pub fn group_join(
    ipc_tx:                 oneshot::Sender<Response>,
    team_id:                String,
    status:                 State,
    rpc_command_tx:         mpsc::Sender<RpcEvent>,
    tunnel_command_tx:      mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>,
    daemon_event_tx:        mpsc::Sender<DaemonEvent>,
) {
    info!("is_not_proxy");
    let _ = is_not_proxy(ipc_tx)
        .and_then(|ipc_tx|{
            info!("is_rpc_connected");
            is_rpc_connected(ipc_tx, &status)
        })
        .and_then(|ipc_tx|{
//            info!("is_joined");
//            if is_joined(&team_id) {
//                Some(ipc_tx)
//            }
//            else {
            info!("send_rpc_join_group");
            send_rpc_join_group(&team_id, ipc_tx, rpc_command_tx.clone())
//            }
        })
        .and_then(|ipc_tx| {
            info!("add_start_team");
            add_start_team(&team_id);
            info!("need_tunnel_connect");
            if need_tunnel_connect() {
                info!("handle_connect_select_proxy");
                handle_connect_select_proxy(ipc_tx, rpc_command_tx)
                    .and_then(|ipc_tx| {
                        info!("handle_tunnel_connect");
                        handle_tunnel_connect(ipc_tx, tunnel_command_tx, daemon_event_tx)
                    })
            }
            else {
                Some(ipc_tx)
            }
        })
        .and_then(|ipc_tx| {
            let response = Response::success();
            let _ = Daemon::oneshot_send(ipc_tx, response, "");
            info!("success");
            Some(())
        });
}

fn is_not_proxy(ipc_tx: oneshot::Sender<Response>) -> Option<oneshot::Sender<Response>> {
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

fn is_rpc_connected(
    ipc_tx:   oneshot::Sender<Response>,
    status:   &State,
) -> Option<oneshot::Sender<Response>> {
    if status.rpc == RpcState::Connected {
        return Some(ipc_tx)
    }
    else {
        let response = Response::internal_error().set_msg("NotLogIn.".to_owned());
        let _ = Daemon::oneshot_send(ipc_tx, response, "");
        return None;
    }
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

fn need_tunnel_connect() -> bool {
    let info = get_info().lock().unwrap();
    if info.teams.running_teams.len() == 1 {
       return true;
    }
    else {
        return false;
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
    daemon_event_tx:    mpsc::Sender<DaemonEvent>,
) -> Option<oneshot::Sender<Response>> {
    let response = send_tunnel_connect(tunnel_command_tx, daemon_event_tx);
    if response.code == 200 {
        return Some(ipc_tx);
    }
    else {
        let _ = Daemon::oneshot_send(ipc_tx, response, "");
        return None;
    }
}