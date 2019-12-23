use std::fmt;
use std::net::IpAddr;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Team {
    pub company_id:         Option<String>,
    pub create_by:          Option<String>,
    pub create_time:        Option<String>,
    pub enable_flag:        Option<bool>,
    pub private_team_flag:  Option<bool>,
    pub site_count:         Option<u32>,
    pub team_des:           Option<String>,
    pub team_id:            String,
    pub members:            Vec<TeamMember>,
    pub team_name:          Option<String>,
    pub terminal_count:     Option<u32>,
    pub update_by:          Option<String>,
    pub update_time:        Option<String>,
    pub user_count:         Option<u32>,
    pub user_id:            Option<String>,
}

impl Team {
    pub fn new() -> Self {
        Team {
            company_id:         None,
            create_by:          None,
            create_time:        None,
            enable_flag:        None,
            private_team_flag:  None,
            site_count:         None,
            team_des:           None,
            team_id:            String::new(),
            members:            vec![],
            team_name:          None,
            terminal_count:     None,
            update_by:          None,
            update_time:        None,
            user_count:         None,
            user_id:            None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TeamMember {
    pub alias:                         Option<String>,
    pub app_version:                   Option<String>,
    pub city:                          Option<String>,
    pub company_id:                    Option<String>,
    pub connection_limit:              Option<String>,
    pub country:                       Option<String>,
    pub create_by:                     Option<String>,
    pub create_time:                   Option<String>,
    pub device_name:                   Option<String>,
    pub device_serial:                 String,
    pub device_type:                   Option<i8>,
    pub hidden_flag:                   Option<bool>,
    pub id:                            Option<String>,
    pub vip:                           IpAddr,
    pub lan:                           Vec<NetSegment>,
    pub latitude:                      Option<String>,
    pub longitude:                     Option<String>,
    pub pubkey:                        String,
    pub region:                        Option<String>,
    // is connect with proxy?
    pub tinc_status:                   bool,
    // is allow connect in this team?
    pub connect_status:                bool,
    pub update_by:                     Option<String>,
    pub update_time:                   Option<String>,
    pub username:                      Option<String>,
    pub wan:                           Option<String>,
    pub is_self:                       Option<bool>,
    // is local tinc host up?
    pub is_local_tinc_host_up:         Option<bool>,
}

// mask CIDR.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct NetSegment {
    pub ip:     IpAddr,
    pub mask:   u32,
    pub gw:     Option<IpAddr>,
}

impl NetSegment {
    pub fn new(ip: IpAddr, mask: u32, gw: Option<IpAddr>) -> Self {
        Self {
            ip,
            mask,
            gw,
        }
    }

    pub fn to_string(&self) -> String {
        if let Some(gw) = self.gw {
            format!("{}/{}/{}", self.ip.to_string(), self.mask, gw.to_string())
        }
        else {
            format!("{}/{}", self.ip.to_string(), self.mask)
        }
    }
}

impl fmt::Debug for NetSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if let Some(gw) = self.gw {
            write!(f, "{}/{}/{}", self.ip.to_string(), self.mask, gw.to_string())
        }
        else {
            write!(f, "{}/{}", self.ip.to_string(), self.mask)
        }
    }
}