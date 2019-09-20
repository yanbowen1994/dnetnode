use crate::info::team::{Team, TeamMember, DeviceProxy};

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaResponseTeamSearch {
    pub code:               Option<u32>,
    pub data:               Option<Vec<JavaResponseTeam>>,
    pub msg:                Option<String>,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaResponseTeam {
    pub enable:             Option<bool>,
    pub members:            Vec<JavaResponseTeamMember>,
    pub siteCount:          Option<u32>,
    pub teamDes:            Option<String>,
    pub teamId:             Option<String>,
    pub teamName:           Option<String>,
    pub terminalCount:      Option<u32>,
    pub userCount:          Option<u32>,
    pub userId:             Option<String>,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaResponseTeamMember {
    pub appversion:         Option<String>,
    pub city:               Option<String>,
    pub connectionLimit:    Option<String>,
    pub country:            Option<String>,
    pub ip:                 Option<String>,
    pub labelName:          Option<String>,
    pub lan:                Option<String>,
    pub latitude:           Option<String>,
    pub longitude:          Option<String>,
    pub mac:                Option<String>,
    pub memberPublicKey:    Option<String>,
    pub memberType:         Option<u32>,
    pub proxylist:          Vec<JavaResponseDeviceProxy>,
    pub pubkey:             Option<String>,
    pub region:             Option<String>,
    pub serviceaddress:     Option<String>,
    pub status:             Option<u32>,
    pub teamId:             Option<String>,
    pub userId:             Option<String>,
    pub userName:           Option<String>,
    pub wan:                Option<String>,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaResponseDeviceProxy {
    pub city:               Option<String>,
    pub country:            Option<String>,
    pub proxyIp:            Option<String>,
    pub proxygw:            Option<String>,
    pub proxypubkey:        Option<String>,
}

impl Into<Team> for JavaResponseTeam {
    fn into(mut self) -> Team {
        let members: Vec<TeamMember> = self.members
            .iter_mut()
            .map(|member|member.clone().into())
            .collect();
        Team {
            enable:          self.enable.unwrap_or(false),
            members,
            site_count:      self.siteCount.unwrap_or(0),
            team_des:        self.teamDes.unwrap_or(String::new()),
            team_id:         self.teamId.unwrap_or(String::new()),
            team_name:       self.teamName.unwrap_or(String::new()),
            terminal_count:  self.terminalCount.unwrap_or(0),
            user_count:      self.userCount.unwrap_or(0),
            user_id:         self.userId.unwrap_or(String::new()),
        }
    }
}

impl Into<TeamMember> for JavaResponseTeamMember {
    fn into(mut self) -> TeamMember {
        let proxylist: Vec<DeviceProxy> = self.proxylist
            .iter_mut()
            .map(|proxy|proxy.clone().into())
            .collect();
        TeamMember {
            appversion:         self.appversion.unwrap_or(String::new()),
            city:               self.city.unwrap_or(String::new()),
            connection_limit:   self.connectionLimit.unwrap_or(String::new()),
            country:            self.country.unwrap_or(String::new()),
            ip:                 self.ip.unwrap_or(String::new()),
            label_name:         self.labelName.unwrap_or(String::new()),
            lan:                self.lan.unwrap_or(String::new()),
            latitude:           self.latitude.unwrap_or(String::new()),
            longitude:          self.longitude.unwrap_or(String::new()),
            mac:                self.mac.unwrap_or(String::new()),
            member_public_key:  self.memberPublicKey.unwrap_or(String::new()),
            member_type:        self.memberType.unwrap_or(0),
            proxylist,
            pubkey:             self.pubkey.unwrap_or(String::new()),
            region:             self.region.unwrap_or(String::new()),
            serviceaddress:     self.serviceaddress.unwrap_or(String::new()),
            status:             self.status.unwrap_or(0),
            team_id:            self.teamId.unwrap_or(String::new()),
            user_id:            self.userId.unwrap_or(String::new()),
            user_name:          self.userName.unwrap_or(String::new()),
            wan:                self.wan.unwrap_or(String::new()),
        }
    }
}

impl Into<DeviceProxy> for JavaResponseDeviceProxy {
    fn into(self) -> DeviceProxy {
        DeviceProxy {
            city:          self.city.unwrap_or(String::new()),
            country:       self.country.unwrap_or(String::new()),
            proxy_ip:      self.proxyIp.unwrap_or(String::new()),
            proxygw:       self.proxygw.unwrap_or(String::new()),
            proxypubkey:   self.proxypubkey.unwrap_or(String::new()),
        }
    }
}
