use std::collections::HashMap;

use dnet_types::team::Team;
use sandbox::route;

use crate::settings::get_settings;
use crate::rpc::http_request::get;
use crate::rpc::{Result, Error};
use crate::info::{get_info, get_mut_info};
use super::super::types::ResponseTeam;
use std::net::IpAddr;
use crate::settings::default_settings::TINC_INTERFACE;

pub fn search_team_by_mac() -> Result<()> {
    let info = get_info().lock().unwrap();
    let device_id = info.client_info.device_name.clone();
    std::mem::drop(info);
    let url = get_settings().common.conductor_url.clone()
        + "/vlan/team/queryByDeviceSerial?deviceSerial=" + &device_id;

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

    info!("{:?}", teams_vec);

    let mut teams = HashMap::new();
    for team in teams_vec {
        teams.insert(team.team_id.clone(), team);
    }

    let mut info = get_mut_info().lock().unwrap();
    info.teams.all_teams = teams;
    fresh_route();
    Ok(())
}

fn fresh_route() {
    let mut connect_client: Vec<IpAddr> = vec![];

    let info = get_info().lock().unwrap();
    let self_vip = match info.tinc_info.vip {
        Some(x) => x,
        None => return,
    };
    let running_team = &info.teams.running_teams;
    for (team_id, team) in &info.teams.all_teams {
        if running_team.contains(team_id) {
            let mut this_team_connect_client = team.members.iter()
                .filter_map(|member| {
                    if member.vip != self_vip && member.status == 1 {
                        Some(member.vip.clone())
                    }
                    else {
                        None
                    }
                })
                .collect::<Vec<IpAddr>>();
            connect_client.append(&mut this_team_connect_client);
        }
    }


    let now_route = route::parse_routing_table();

    for client_vip in &connect_client {
        if !route::is_in_routing_table(
            &now_route,
            client_vip,
            32,
            TINC_INTERFACE) {
            route::add_route(client_vip, 32, TINC_INTERFACE)
        }
    }

}


