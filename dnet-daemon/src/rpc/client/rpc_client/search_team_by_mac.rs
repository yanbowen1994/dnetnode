use std::net::IpAddr;
use std::collections::HashMap;

use sandbox::route;

use crate::settings::get_settings;
use crate::rpc::{Result, Error};
use crate::info::{get_info, get_mut_info};
use super::types::ResponseTeam;
use crate::settings::default_settings::TINC_INTERFACE;
use crate::rpc::http_request::{get, MAX_PAGE, PAGESIZE, get_records};
use dnet_types::team::Team;

pub fn search_team_by_mac() -> Result<()> {
    let info = get_info().lock().unwrap();
    let device_id = info.client_info.device_name.clone();
    std::mem::drop(info);
    let url = get_settings().common.conductor_url.clone()
        + "/vlan/team/queryByDeviceSerial?deviceSerial=" + &device_id;

    search_team_inner(url)?;

    Ok(())
}

pub fn search_team_inner(mut url: String) -> Result<()> {
    let mut teams_vec: Vec<Team> = vec![];
    for i in 0..MAX_PAGE {
        url = url + &format!("&pageNum={}&pageSize={}", i, PAGESIZE);
        let recv = get(&url)?;
        let recv = get_records(&url, recv)?;

        if recv.len() < PAGESIZE {
            teams_vec.append(&mut parse_to_team(recv)?);
            break
        } else {
            teams_vec.append(&mut parse_to_team(recv)?);
        }
    }

    log_of_team(&teams_vec);

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
    let _ = std::thread::Builder::new()
        .name("keep_route".to_string())
        .spawn(move ||
            route::keep_route(local_vip, hosts, TINC_INTERFACE.to_string())
        );
    Ok(())
}

fn log_of_team(teams_vec: &Vec<Team>) {
    let mut connect_id_member: HashMap<String, Vec<&IpAddr>> = HashMap::new();
    let mut disconnect_id_member: HashMap<String, Vec<&IpAddr>> = HashMap::new();
    for team in teams_vec {
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
}

fn parse_to_team(res_data: Vec<serde_json::Value>) -> Result<Vec<Team>> {
    let mut teams_vec = vec![];
    for team in res_data {
        let res_team = serde_json::from_value::<ResponseTeam>(team.clone())
            .map_err(|err| {
                error!("parse team info failed.err: {:?} {:?}", err, team);
                Error::ResponseParse("GetProxyResponse".to_string())
            })?;
        teams_vec.push(res_team.parse_to_team())
    }
    Ok(teams_vec)
}