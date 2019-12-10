use dnet_types::team::{
    Team as DaemonTeam,
    TeamMember as DaemonTeamMember,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamStatusResponse {
    pub code:               u32,
    pub teams:              Vec<Team>,
}


impl From<Vec<DaemonTeam>> for TeamStatusResponse {
    fn from(teams: Vec<DaemonTeam>) -> Self {
        let teams = teams
            .iter()
            .map(|daemon_team|Team::from(daemon_team.to_owned()))
            .collect();
        Self {
            code: 200,
            teams,
        }
    }
}

impl TeamStatusResponse {
    pub fn to_json_str(&self) -> String {
        return serde_json::to_string(&self).unwrap();
    }
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    id:                 String,
    name:               String,
    offline:            u32,
    online:             u32,
    members:            Vec<Member>,
}

impl From<DaemonTeam> for Team {
    fn from(team: DaemonTeam) -> Self {
        let mut offline = 0;
        let mut online = 0;

        let mut members = vec![];
        for source_member in team.members {
            let member = Member::from(source_member);
            members.push(member.clone());

            if member.status == false {
                offline += 1;
            }
            else {
                online += 1;
            }
        }


        Self {
            id:        team.team_id,
            name:      team.team_name.unwrap_or("".to_string()),
            offline,
            online,
            members,
        }
    }
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Member {
    id:         String,
    status:     bool,
    serial_num: String,
    ip:         String,
    wan_ip:     String,
    sub_ip:     String,
    svtype:     String,
}

impl From<DaemonTeamMember> for Member {
    fn from(member: DaemonTeamMember) -> Self {
        let lans = member.lan.iter()
            .map(|lan| {
                lan.to_string()
            })
            .collect::<Vec<String>>();
        let sub_ip = format!("{:?}", member.lan);
        let svtype = format!("{}", member.device_type.unwrap_or(6));
        Self {
            id:         member.device_name.unwrap_or("".to_string()),
            status:     member.tinc_status && member.connect_status,
            serial_num: member.device_serial,
            ip:         member.vip.to_string(),
            wan_ip:     member.wan.unwrap_or("".to_string()),
            sub_ip,
            svtype,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Lan {
    lan_name:           String,
    lan_subnet:         String,
}