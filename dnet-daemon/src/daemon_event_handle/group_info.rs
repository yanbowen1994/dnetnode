use std::sync::mpsc;
use std::time::Duration;

use futures::sync::oneshot;

use dnet_types::response::Response;
use crate::rpc::rpc_cmd::{RpcEvent, RpcClientCmd};
use crate::daemon::Daemon;
use crate::info::get_info;
use dnet_types::team::Team;

pub fn handle_group_info(
    ipc_tx:                 oneshot::Sender<Vec<Team>>,
    rpc_command_tx:         mpsc::Sender<RpcEvent>,
    team_id:                Option<String>
) {
    info!("send_rpc_group_fresh");
    let res = send_rpc_group_fresh(rpc_command_tx);

    if res.code == 200 {
        if let Some(team_id) = team_id {
            if let Some(team) = get_info().lock().unwrap()
                .teams
                .all_teams
                .get(&team_id)
                .cloned() {
                let _ = Daemon::oneshot_send(ipc_tx, vec![team], "");
            }
            else {
                let _ = Daemon::oneshot_send(ipc_tx, vec![], "");
            }
        }
        else {
            let team =  get_info().lock().unwrap()
                .teams
                .all_teams
                .values()
                .cloned()
                .collect::<Vec<Team>>();
            let _ = Daemon::oneshot_send(ipc_tx, team, "");
        }
    }
    else {
        let response = vec![];
        let _ = Daemon::oneshot_send(ipc_tx, response, "");
    }
}

fn send_rpc_group_fresh(
    rpc_command_tx:     mpsc::Sender<RpcEvent>
) -> Response {
    let (res_tx, res_rx) = mpsc::channel();
    let _ = rpc_command_tx.send(RpcEvent::Client(RpcClientCmd::FreshTeam(res_tx)));
    if let Ok(res) = res_rx.recv_timeout(Duration::from_secs(5)) {
        res
    }
    else {
        Response::exec_timeout()
    }
}