use std::collections::HashMap;
use dnet_types::team::Team;
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

    pub fn get_connect_hosts(&self,
                             self_vip: &Option<IpAddr>) -> Vec<IpAddr> {
        let mut connects: Vec<IpAddr> = vec![];
        if let Some(self_vip) = self_vip {
            for team_id in &self.running_teams {
                if let Some(team) = self.all_teams.get(team_id) {
                    for member in &team.members {
                        if member.connect_status == true && member.vip != *self_vip {
                            connects.push(member.vip.clone())
                        }
                    }
                }
            }
        }
        connects
    }
}