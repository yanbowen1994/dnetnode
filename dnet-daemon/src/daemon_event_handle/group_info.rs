use std::sync::mpsc;

use futures::sync::oneshot;
use dnet_types::response::Response;
use dnet_types::team::Team;

use crate::rpc::rpc_cmd::RpcEvent;
use crate::daemon::Daemon;
use crate::info::get_info;
use crate::daemon_event_handle::common::{send_rpc_group_fresh, is_rpc_connected};

pub fn handle_group_info(
    ipc_tx:                 oneshot::Sender<Response>,
    rpc_command_tx:         mpsc::Sender<RpcEvent>,
    team_id:                Option<String>,
) {
    if let Some(ipc_tx) = is_rpc_connected(ipc_tx) {
        info!("send_rpc_group_fresh");
        let res = send_rpc_group_fresh(rpc_command_tx);

        if res.code == 200 {
            if let Some(team_id) = team_id {
                let teams = get_info().lock().unwrap().get_current_team_connect();
                let team = teams.into_iter()
                    .filter_map(|team| {
                        if team.team_id == team_id {
                            Some(team)
                        }
                        else {
                            None
                        }
                    })
                    .collect::<Vec<Team>>();
                if let Ok(data) = serde_json::to_value(team) {
                    let res = Response::success().set_data(Some(data));
                    let _ = Daemon::oneshot_send(ipc_tx, res, "");
                    return;
                }
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