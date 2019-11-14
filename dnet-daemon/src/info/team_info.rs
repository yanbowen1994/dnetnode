use std::collections::HashMap;
use dnet_types::team::Team;
use sandbox::route::del_route;
use crate::settings::default_settings::TINC_INTERFACE;

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
}