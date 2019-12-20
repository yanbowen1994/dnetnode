use futures::sync::oneshot;
use dnet_types::response::Response;
use dnet_types::team::Team;

use crate::daemon::Daemon;
use crate::info::get_info;
use crate::daemon_event_handle::common::is_not_proxy;

pub fn handle_group_info(
    ipc_tx:                 oneshot::Sender<Response>,
    team_id:                Option<String>,
) {
    let ipc_tx = match is_not_proxy(ipc_tx) {
        Some(x) => x,
        None => return,
    };

    if let Some(team_id) = team_id {
        let teams = get_info().lock().unwrap().get_current_team_connect();
        let mut team = teams.into_iter()
            .filter_map(|team| {
                if team.team_id == team_id {
                    Some(team)
                }
                else {
                    None
                }
            })
            .collect::<Vec<Team>>();

        teams_sort(&mut team);
        if let Ok(data) = serde_json::to_value(team) {
            let res = Response::success().set_data(Some(data));
            let _ = Daemon::oneshot_send(ipc_tx, res, "");
            return;
        }
    }
    else {
        let mut teams = get_info().lock().unwrap().get_current_team_connect();
        teams_sort(&mut teams);

        if let Ok(data) = serde_json::to_value(teams) {
            let res = Response::success().set_data(Some(data));
            let _ = Daemon::oneshot_send(ipc_tx, res, "");
            return;
        }
    }
}

fn teams_sort(teams: &mut Vec<Team>) {
    teams.sort_by(|a, b|a.team_name.cmp(&b.team_name));
    let _ = teams
        .iter_mut()
        .map(|team| {
            team.members.sort_by(|a, b|a.vip.cmp(&b.vip));
            let _ = team.members
                .iter_mut()
                .map(|mut member| {
                    member.pubkey = String::new();
                })
                .collect::<Vec<()>>();
        })
        .collect::<Vec<()>>();
}