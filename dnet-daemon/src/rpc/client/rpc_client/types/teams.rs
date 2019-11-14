use std::net::IpAddr;
use std::str::FromStr;
use dnet_types::team::{Team, TeamMember, NetSegment};
use dnet_types::device_type::DeviceType;
use crate::info::get_info;

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
    pub members:            Option<Vec<JavaResponseTeamMember>>,
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
    pub proxylist:          Option<Vec<JavaResponseDeviceProxy>>,
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
    fn into(self) -> Team {
        let members: Vec<TeamMember> = self.members
            .map(|mut members|
                members
                    .iter_mut()
                    .filter_map(|member|member.clone().into())
                    .collect::<Vec<TeamMember>>()
            )
            .unwrap_or(vec![]);
        Team {
            members,
            team_id:         self.teamId.unwrap_or(String::new()),
            team_name:       self.teamName.unwrap_or(String::new()),
        }
    }
}

impl JavaResponseTeamMember {
    fn into(self) -> Option<TeamMember> {
        let device_type = self.memberType
            .map(|type_code|DeviceType::from(type_code))
            .unwrap_or(DeviceType::Other);

        let lan = self.lan
            .map(|lan_str| {
                let lans: Vec<NetSegment> = lan_str
                    .split(",")
                    .collect::<Vec<&str>>()
                    .iter_mut()
                    .filter_map(|res_str| {
                        let ip_mask: Vec<&str> = res_str.split("/").collect();
                        if ip_mask.len() == 2 {
                            if let Ok(ip) = IpAddr::from_str(ip_mask[0]) {
                                if let Ok(mask_cidr) = ip_mask[0].parse::<u32>() {
                                    return Some(NetSegment {
                                        ip,
                                        mask: mask_cidr,
                                    });
                                }
                            }
                        }
                        return None;
                    })
                    .collect();
                lans
            })
            .unwrap_or(vec![]);

        let proxy_ip: Vec<IpAddr> = self.proxylist
            .unwrap_or(vec![])
            .iter()
            .filter_map(|device_proxy|
                            device_proxy.proxyIp
                                .to_owned()
                                .and_then(|ip_str|
                                    IpAddr::from_str(&ip_str)
                                        .map(|ip| Some(ip))
                                        .unwrap_or(None)
                                ))
            .collect();

        let vip = self.ip
            .and_then(|ip_str|
                IpAddr::from_str(&ip_str).ok()
            );

        if vip.is_none() {
            return None;
        }

        let info = get_info().lock().unwrap();

        let is_self = if self.mac.as_ref() == Some(&info.client_info.devicename) {
            true
        }
        else {
            false
        };

        Some(TeamMember {
            device_id:      self.mac.clone().unwrap_or("".to_owned()),
            device_name:    self.labelName.unwrap_or(self.mac.unwrap_or("".to_owned())),
            device_type,
            vip:            vip.unwrap(),
            lan,
            wan:            self.wan.unwrap_or("".to_string()),
            proxy_ip,
            status:         self.status.unwrap_or(0),
            is_self,
        })
    }
}