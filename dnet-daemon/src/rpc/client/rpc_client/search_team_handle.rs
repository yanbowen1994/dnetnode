use dnet_types::team::Team;

use crate::info::{get_mut_info, get_info};
use super::types::teams::JavaResponseTeam;
use super::Result;
use std::collections::HashMap;

// if return true restart tunnel.
pub fn search_team_handle(mut jteams: Vec<JavaResponseTeam>) -> Result<bool> {
    let info = get_info().lock().unwrap();
    let device_id = info.client_info.uid.clone();
    std::mem::drop(info);

    let teams: Vec<Team> = jteams
        .iter_mut()
        .map(|jteam| jteam.clone().into())
        .collect();

    let mut local_vip = None;

    let mut all_teams = HashMap::new();

    for team in &teams {
        println!("{:?}", team);
        for member in &team.members {
            if &member.device_id == &device_id {
                local_vip = Some(member.vip.clone());
            }
        }
        all_teams.insert(team.team_id.clone(), team.clone());
    }


    let mut info = get_mut_info().lock().unwrap();
    info.teams.all_teams = all_teams;
    if local_vip != None {
        if info.tinc_info.vip != local_vip {
            info.tinc_info.vip = local_vip;
            return Ok(true);
        }
    }
    Ok(false)
}