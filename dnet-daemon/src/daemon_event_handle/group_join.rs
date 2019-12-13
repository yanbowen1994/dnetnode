use std::sync::mpsc;
use std::time::Duration;

use futures::sync::oneshot;

use dnet_types::response::Response;

use crate::rpc::rpc_cmd::{RpcEvent, RpcClientCmd};
use crate::daemon::Daemon;
use super::handle_settings;
use super::common::is_not_proxy;
use crate::daemon_event_handle::common::{is_rpc_connected, send_rpc_group_fresh, daemon_event_handle_fresh_running_from_all};

pub fn group_join(
    ipc_tx:                 oneshot::Sender<Response>,
    team_id:                String,
    rpc_command_tx:         mpsc::Sender<RpcEvent>,
) {
    info!("is_not_proxy");
    let _ = is_not_proxy(ipc_tx)
        .and_then(|ipc_tx| {
            info!("check_conductor_url");
            handle_settings::check_conductor_url(ipc_tx)
        })
        .and_then(|ipc_tx|{
            info!("is_rpc_connected");
            is_rpc_connected(ipc_tx)
        })
        .and_then(|ipc_tx|{
            info!("send_rpc_join_group");
            send_rpc_join_group(&team_id, ipc_tx, rpc_command_tx.clone())
        })
        .and_then(|ipc_tx| {
            let response = send_rpc_group_fresh(rpc_command_tx);
            if response.code == 200{
                daemon_event_handle_fresh_running_from_all();
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