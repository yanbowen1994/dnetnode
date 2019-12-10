use std::collections::HashMap;
use dnet_types::team::Team;
use sandbox::route::del_route;
use crate::settings::default_settings::TINC_INTERFACE;
use tinc_plugin::{TincTeam, TincTools};
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct TeamInfo {
    pub all_teams:              HashMap<String, Team>,
    pub running_teams:          Vec<String>,
}

impl TeamInfo {
    pub fn new() -> Self {
        Self {
            all_teams:      HashMap::new(),
            running_teams:  vec![],
        }
    }

    pub fn add_start_team(&mut self, team_id: &str) -> bool {
        let is_contains = self.all_teams.contains_key(team_id);
        if is_contains {
            self.running_teams.push(team_id.to_owned());
        }
        is_contains
    }

    pub fn del_start_team(&mut self, team_id: &str) {
        for i in 0..self.running_teams.len() {
            if &self.running_teams[i] == team_id {
                self.running_teams.remove(i);
                break
            }
        }
        let team = self.all_teams.get(team_id);
        if let Some(team) = team {
            self.del_start_team_route(team);
        }
    }

    fn del_start_team_route(&self, team: &Team) {
        for memeber in &team.members {
            del_route(&memeber.vip, 32, TINC_INTERFACE);
        }
    }

    pub fn to_tinc_team(&self) -> TincTeam {
        let mut add = HashMap::new();

        for (team_id, team) in &self.all_teams {
            let vips = team.members
                .iter()
                .map(|member| {
                    member.vip.clone()
                })
                .collect::<Vec<IpAddr>>();
            add.insert(team_id.to_string(), vips);
        }

        TincTeam {
            add,
            delete: HashMap::new(),
        }
    }

    /// return (device_name, in which running team's team_id)
    pub fn find_host_in_running(&self, host: &str) -> (String, Vec<String>) {
        let all_teams = &self.all_teams;
        let mut team_id_vec: Vec<String> = vec![];
        let mut device_name = String::new();
        for (team_id, team) in all_teams {
            if self.running_teams.contains(team_id) {
                for member in &team.members {
                    if let Some(find_device_name) =
                        if Some(member.vip) == TincTools::get_vip_by_filename(host) {
                            member.device_name.clone()
                        }
                        else {
                            continue
                        } {

                        if device_name.is_empty() {
                            device_name = find_device_name;
                        }

                        team_id_vec.push(team_id.to_owned());
                        break
                    }
                }
            }
        }
        (device_name, team_id_vec)
    }

    pub fn get_connect_hosts<'a>(&self,
                                 teams: &'a HashMap<String, Team>,
                                 self_vip: &IpAddr) -> Vec<&'a IpAddr> {
        let mut connects: Vec<&IpAddr> = vec![];
        for team_id in &self.running_teams {
            if let Some(team) = teams.get(team_id) {
                for member in &team.members {
                    if member.connect_status == 1 && member.vip != *self_vip {
                        connects.push(&member.vip)
                    }
                }
            }
        }
        connects
    }
}