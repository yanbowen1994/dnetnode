use std::collections::HashMap;
use std::net::IpAddr;

use dnet_types::team::Team;
use sandbox::route;

use crate::info::get_mut_info;
use crate::settings::get_settings;
use crate::rpc::http_request::get;
use crate::rpc::{Error, Result};
use crate::rpc::client::rpc_client::types::ResponseTeam;
use crate::settings::default_settings::TINC_INTERFACE;

pub(super) fn search_team_by_user() -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vlan/team/queryMyAll";

    let res_data = get(&url)?;

    let teams_vec = res_data.as_array()
        .and_then(|team_vec| {
            let team_vec = team_vec.clone();
            Some(team_vec
                .into_iter()
                .filter_map(|team| {
                    let res_team = serde_json::from_value::<ResponseTeam>(team.clone())
                        .map_err(|err| {
                            error!("parse team info failed.err: {:?} {:?}", err, team);
                        })
                        .ok()?;
                    Some(res_team.parse_to_team())
                })
                .collect::<Vec<Team>>())
        })
        .ok_or(Error::ResponseParse(res_data.to_string()))?;

    let mut id_member: HashMap<String, Vec<&IpAddr>> = HashMap::new();
    for team in &teams_vec {
        let mut members = vec![];
        for member in &team.members {
            if member.connect_status == true {
                members.push(&member.vip)
            }
        }
        id_member.insert(team.team_id.clone(), members);
    }
    info!("{:?}", id_member);

    let mut teams = HashMap::new();
    for team in teams_vec {
        teams.insert(team.team_id.clone(), team);
    }

    let mut info = get_mut_info().lock().unwrap();
    info.teams.all_teams = teams;
    info.fresh_running_from_all();
    let hosts = info.teams.get_connect_hosts(&info.tinc_info.vip);
    let local_vip = info.tinc_info.vip.clone();
    std::mem::drop(info);
    info!("route hosts {:?}", hosts);
    std::thread::spawn(move ||
        route::keep_route(local_vip, hosts, TINC_INTERFACE.to_string())
    );
    Ok(())
}