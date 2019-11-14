use std::net::IpAddr;
use crate::device_type::DeviceType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    pub team_id:    String,
    pub team_name:  String,
    pub members:    Vec<TeamMember>,
}

impl Team {
    pub fn new(team_id: &str, team_name: &str, members: Vec<TeamMember>) -> Self {
        Team {
            team_id: team_id.to_owned(),
            team_name: team_name.to_owned(),
            members,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamMember {
    pub device_id:      String,
    pub device_name:    String,
    pub device_type:    DeviceType,
    pub vip:            IpAddr,
    pub lan:            Vec<NetSegment>,
    pub wan:            String,
    pub proxy_ip:       Vec<IpAddr>,
    pub status:         u32,
    pub is_self:        bool,
}

// mask CIDR.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NetSegment {
    pub ip:     IpAddr,
    pub mask:   u32,
}

impl NetSegment {
    pub fn to_string(&self) -> String {
        format!("{}/{}", self.ip.to_string(), self.mask)
    }
}

//#[derive(Clone, Debug, Serialize, Deserialize)]
//pub struct Team {
//    pub enable:             bool,
//    pub members:            Vec<TeamMember>,
//    pub site_count:         u32,
//    pub team_des:           String,
//    pub team_id:            String,
//    pub team_name:          String,
//    pub proxy_ip:           String,
//    pub subnet:             String,
//    pub terminal_count:     u32,
//    pub user_count:         u32,
//    pub user_id:            String,
//}
//
//impl Team {
//    pub fn new(members: Vec<TeamMember>, team_id: &str, team_name: &str) -> Self {
//        Self {
//            enable:             true,
//            members,
//            site_count:         0,
//            team_des:           String::new(),
//            team_id:            team_id.to_owned(),
//            team_name:          team_name.to_owned(),
//            proxy_ip:           String::new(),
//            subnet:             String::new(),
//            terminal_count:     0,
//            user_count:         0,
//            user_id:            String::new(),
//        }
//    }
//}
//
//#[derive(Clone, Debug, Serialize, Deserialize)]
//pub struct TeamMember {
//    pub appversion:         String,
//    pub city:               String,
//    pub connection_limit:   String,
//    pub country:            String,
//    pub ip:                 String,
//    pub label_name:         String,
//    pub lan:                Vec<String>,
//    pub latitude:           String,
//    pub longitude:          String,
//    pub mac:                String,
//    pub member_public_key:  String,
//    pub member_type:        u32,
//    pub proxylist:          Vec<DeviceProxy>,
//    pub pubkey:             String,
//    pub region:             String,
//    pub serviceaddress:     String,
//    pub status:             u32,
//    pub team_id:            String,
//    pub user_id:            String,
//    pub user_name:          String,
//    pub wan:                String,
//}
//
//#[derive(Clone, Debug, Serialize, Deserialize)]
//pub struct DeviceProxy {
//    pub city:               String,
//    pub country:            String,
//    pub proxy_ip:           String,
//    pub proxygw:            String,
//    pub proxypubkey:        String,
//}