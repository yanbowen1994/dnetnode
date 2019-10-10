use std::net::IpAddr;
use std::str::FromStr;

use dnet_types::team::Team;

use super::types::teams::JavaResponseTeam;
use super::{Error, Result};
use crate::info::{get_mut_info, get_info};

pub fn search_team_handle(mut jteams: Vec<JavaResponseTeam>) -> Result<()> {
    let mut info = get_info().lock().unwrap();
    let device_id = info.client_info.uid.clone();
    std::mem::drop(info);

    let teams: Vec<Team> = jteams
        .iter_mut()
        .map(|jteam| jteam.clone().into())
        .collect();

    let mut local_vip = None;

    for team in &teams {
        println!("{:?}", team);
        for member in &team.members {
            if &member.mac == &device_id {
                let vip = IpAddr::from_str(&member.ip)
                    .map_err(|e| {
                        error!("search_team_by_mac can't parse self vip.");
                        Error::ParseIp(e)
                    })?;

                local_vip = Some(vip);
            }
        }
    }

    let mut info = get_mut_info().lock().unwrap();
    info.teams = teams;
    info.tinc_info.vip = local_vip;
    Ok(())
}