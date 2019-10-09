use dnet_types::team::{
    Team as DaemonTeam,
    TeamMember as DaemonTeamMember,
    DeviceProxy,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamStatusResponse {
    code:               String,
    teams:              Vec<Team>,
    on:                 String,
    cloud_led_on:       String,
    sn:                 String,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Team {
    teamName:           String,
    teamDes:            String,
    proxyIp:            String,
    subnet:	            String,
    userId:	            String,
    siteCount:	        u32,
    terminalCount:	    u32,
    teamId:	            String,
    pubKey:	            String,
    teamUserId:         String,
    members:            Vec<Member>,
}

impl From<DaemonTeam> for Team {
    fn from(team: DaemonTeam) -> Self {
        let mut members = vec![];
        for source_member in team.members {
            let member = Member::from(source_member);
            members.push(member);
        }

        Self {
            teamName:      team.team_name,
            teamDes:       team.team_des,
            proxyIp:       team.proxy_ip,
            subnet:        team.subnet,
            userId:        team.user_id,
            siteCount:     team.site_count,
            terminalCount: team.terminal_count,
            teamId:        team.team_id,
            pubKey:        String::new(),
            teamUserId:    team.user_id.clone(),
            members,
        }
    }
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Member {
    teamId:             String,
    mac:                String,
    ip:                 String,
    lan:	            Vec<Lan>,
    wan:                String,
    proxyIp:            String,
    labelName:          String,
    status:             u32,
    memberType:	        u32,
    userId:             String,
    connectionLimit:	String,
    geo_ip:             String,
    wan_ip:             String,
    pubkey:             String,
    lan_inf:            String,
    labelName2:         String,
}

impl From<DaemonTeamMember> for Member {
    fn from(member: DaemonTeamMember) -> Self {
        let proxy_ip;
        if member.proxylist.is_empty() {
            proxy_ip = "".to_owned();
        }
        else {
            proxy_ip = member.proxylist[0].proxy_ip.clone();
        }

        let lan = member.lan.clone().iter_mut().map(|lan|
            Lan {
                lan_name: String::new(),
                lan_subnet: lan.to_owned(),
            }).collect::<Vec<Lan>>();

        Self {
            teamId:           member.team_id,
            mac:              member.mac,
            ip:               member.ip,
            lan,
            wan:              member.wan,
            proxyIp:          proxy_ip,
            labelName:        member.label_name,
            status:           member.status,
            memberType:       member.member_type,
            userId:           member.user_id,
            connectionLimit:  member.connection_limit,
            geo_ip:           member.ip.clone(),
            wan_ip:           member.ip.clone(),
            pubkey:           member.pubkey,
            lan_inf:          member.latitude,
            labelName2:       member.label_name.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Lan {
    lan_name:           String,
    lan_subnet:         String,
}