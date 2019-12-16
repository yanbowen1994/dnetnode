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
                let mut team = teams.into_iter()
                    .filter_map(|mut team| {
                        if team.team_id == team_id {
                            team.members.sort_by(|a, b|a.vip.cmp(&b.vip));
                            Some(team)
                        }
                        else {
                            None
                        }
                    })
                    .collect::<Vec<Team>>();

                team.sort_by(|a, b|a.team_name.cmp(&b.team_name));
                if let Ok(data) = serde_json::to_value(team) {
                    let res = Response::success().set_data(Some(data));
                    let _ = Daemon::oneshot_send(ipc_tx, res, "");
                    return;
                }
            }
            else {
                let mut teams = get_info().lock().unwrap().get_current_team_connect();
                let _ = teams.iter_mut()
                    .map(|team|{
                             team.members.sort_by(|a, b|a.vip.cmp(&b.vip));
                    })
                    .collect::<Vec<()>>();

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