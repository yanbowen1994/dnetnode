use std::net::IpAddr;
use std::collections::HashMap;

use dnet_types::team::Team;
use sandbox::route;

use crate::settings::get_settings;
use crate::rpc::http_request::get;
use crate::rpc::{Result, Error};
use crate::info::{get_info, get_mut_info};
use super::types::ResponseTeam;
use crate::settings::default_settings::TINC_INTERFACE;
use crate::rpc::client::rpc_client::select_proxy::ping;

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

    let mut connect_id_member: HashMap<String, Vec<&IpAddr>> = HashMap::new();
    let mut disconnect_id_member: HashMap<String, Vec<&IpAddr>> = HashMap::new();
    for team in &teams_vec {
        let mut connect_members = vec![];
        let mut disconnect_members = vec![];
        for member in &team.members {
            if member.connect_status == true {
                connect_members.push(&member.vip)
            }
            else {
                disconnect_members.push(&member.vip)
            }
        }
        connect_id_member.insert(team.team_id.clone(), connect_members);
        disconnect_id_member.insert(team.team_id.clone(), disconnect_members);
    }
    info!("connect: {:?} disconnect:{:?}", connect_id_member, disconnect_id_member);

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
    for host in &hosts {
        let host = host.clone();
        std::thread::spawn(move ||ping(host));
    }
    std::thread::spawn(move ||
        route::keep_route(local_vip, hosts, TINC_INTERFACE.to_string())
    );
    Ok(())
}