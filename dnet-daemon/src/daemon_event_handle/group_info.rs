use std::sync::mpsc;

use futures::sync::oneshot;
use dnet_types::response::Response;

use crate::rpc::rpc_cmd::RpcEvent;
use crate::daemon::Daemon;
use crate::info::get_info;
use crate::daemon_event_handle::common::{send_rpc_group_fresh, is_rpc_connected};
use dnet_types::states::State;

pub fn handle_group_info(
    ipc_tx:                 oneshot::Sender<Response>,
    rpc_command_tx:         mpsc::Sender<RpcEvent>,
    team_id:                Option<String>,
    status:                 State,
) {
    if let Some(ipc_tx) = is_rpc_connected(ipc_tx, &status) {
        info!("send_rpc_group_fresh");
        let res = send_rpc_group_fresh(rpc_command_tx);

        if res.code == 200 {
            if let Some(team_id) = team_id {
                if let Some(team) = get_info().lock().unwrap()
                    .teams
                    .all_teams
                    .get(&team_id)
                    .cloned() {
                    if let Ok(data) = serde_json::to_value(vec![team]) {
                        let res = Response::success().set_data(Some(data));
                        let _ = Daemon::oneshot_send(ipc_tx, res, "");
                        return;
                    }
                }

                let _ = Daemon::oneshot_send(
                    ipc_tx,
                    Response::internal_error().set_msg("Group Not Found.".to_string()),
                    ""
                );
            }
            else {
                let teams = get_info().lock().unwrap().get_current_team_connect();
                if let Ok(data) = serde_json::to_value(teams) {
                    let res = Response::success().set_data(Some(data));
                    let _ = Daemon::oneshot_send(ipc_tx, res, "");
                    return;
                }
            }
        }
        else {
            let _ = Daemon::oneshot_send(ipc_tx, res, "");
        }
    }
}