use std::collections::HashMap;

use dnet_types::team::Team;

use crate::settings::get_settings;
use crate::rpc::http_request::get;
use crate::rpc::{Result, Error};
use crate::info::{get_info, get_mut_info};
use super::types::ResponseTeam;
use sandbox::route;
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

    let mut teams = HashMap::new();
    for team in teams_vec {
        teams.insert(team.team_id.clone(), team);
    }

    let mut info = get_mut_info().lock().unwrap();
    info.flush_running_team(&teams);
    let (add, del) = info.compare_team_info_with_new_teams(&teams);
    info.teams.all_teams = teams;
    std::mem::drop(info);

    info!("route add {:?}, del {:?}", add, del);
    route::batch_route(&add, &del, TINC_INTERFACE);
    Ok(())
}