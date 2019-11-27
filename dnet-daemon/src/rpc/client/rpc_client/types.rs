use std::net::IpAddr;
use std::str::FromStr;

use dnet_types::team::{TeamMember, Team, NetSegment};
use crate::info::get_mut_info;

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponseTeam {
    pub companyId:         Option<String>,
    pub createBy:          Option<String>,
    pub createTime:        Option<String>,
    pub enableFlag:        Option<i32>,
    pub privateTeamFlag:   Option<bool>,
    pub siteCount:         Option<u32>,
    pub teamDes:           Option<String>,
    pub teamId:            String,
    pub teamMembers:       Vec<ResponseTeamMember>,
    pub teamName:          Option<String>,
    pub terminalCount:     Option<u32>,
    pub updateBy:          Option<String>,
    pub updateTime:        Option<String>,
    pub userCount:         Option<u32>,
    pub userId:            Option<String>,
}

impl ResponseTeam {
    pub fn parse_to_team(self) -> Team {
        let members: Vec<TeamMember> = self.teamMembers.into_iter()
            .filter_map(|team_member| {
                team_member.parse_to_team_member()
            })
            .collect();
        let enable_flag = self.enableFlag.map(|x|{
            if x == 0 {
                true
            }
            else {
                false
            }
        });
        Team {
            company_id:             self.companyId,
            create_by:              self.createBy,
            create_time:            self.createTime,
            enable_flag,
            private_team_flag:      self.privateTeamFlag,
            site_count:             self.siteCount,
            team_des:               self.teamDes,
            team_id:                self.teamId.clone(),
            members,
            team_name:              self.teamName,
            terminal_count:         self.terminalCount,
            update_by:              self.updateBy,
            update_time:            self.updateTime,
            user_count:             self.userCount,
            user_id:                self.userId,
        }
    }
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponseTeamMember {
    pub alias:                Option<String>,
    pub appVersion:           Option<String>,
    pub city:                 Option<String>,
    pub companyId:            Option<String>,
    pub connectionLimit:      Option<String>,
    pub country:              Option<String>,
    pub createBy:             Option<String>,
    pub createTime:           Option<String>,
    pub device_name:           Option<String>,
    pub deviceSerial:         Option<String>,
    pub deviceType:           Option<i8>,
    pub hiddenFlag:           Option<bool>,
    pub id:                   Option<String>,
    pub ip:                   String,
    pub lan:                  Option<String>,
    pub latitude:             Option<String>,
    pub longitude:            Option<String>,
    pub pubKey:               String,
    pub region:               Option<String>,
    pub status:               i8,
    pub updateBy:             Option<String>,
    pub updateTime:           Option<String>,
    pub username:             Option<String>,
    pub wan:                  Option<String>,
}

impl ResponseTeamMember {
    pub fn parse_to_team_member(self) -> Option<TeamMember> {
        let vip = IpAddr::from_str(&self.ip).ok()?;
        let lan: Vec<NetSegment> = serde_json::from_str(&self.ip).ok().unwrap_or(vec![]);
        Some(TeamMember {
            alias:             self.alias,
            app_version:       self.appVersion,
            city:              self.city,
            company_id:        self.companyId,
            connection_limit:  self.connectionLimit,
            country:           self.country,
            create_by:         self.createBy,
            create_time:       self.createTime,
            device_name:       self.device_name,
            device_serial:     self.deviceSerial?,
            device_type:       self.deviceType,
            hidden_flag:       self.hiddenFlag,
            id:                self.id,
            vip,
            lan,
            latitude:          self.latitude,
            longitude:         self.longitude,
            pubkey:            self.pubKey.clone(),
            region:            self.region,
            status:            self.status,
            update_by:         self.updateBy,
            update_time:       self.updateTime,
            username:          self.username,
            wan:               self.wan,
        })
    }
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaDevice {
    pub alias:                      Option<String>,
    pub appVersion:                 Option<String>,
    pub city:                       Option<String>,
    pub companyId:                  Option<String>,
    pub connectionLimit:            Option<String>,
    pub country:                    Option<String>,
    pub createBy:                   Option<String>,
    pub createTime:                 Option<String>,
    pub device_name:                 String,
    pub deviceSerial:               String,
    pub deviceType:                 i8,
    pub hiddenFlag:                 bool,
    pub id:                         Option<String>,
    pub ip:                         Option<String>,
    pub lan:                        Option<String>,
    pub latitude:                   Option<String>,
    pub longitude:                  Option<String>,
    pub pubKey:                     String,
    pub region:                     Option<String>,
    pub tincStatus:                 i8,
    pub updateBy:                   Option<String>,
    pub updateTime:                 Option<String>,
    pub userId:                     Option<String>,
    pub wan:                        Option<String>,
}

impl JavaDevice {
    pub fn new() -> Self {
        let mut info = get_mut_info().lock().unwrap();
        if let Err(e) = info.tinc_info.flush_connections() {
            warn!("{:?}", e);
        }
        let device_name = info.client_info.device_name.clone();
        let device_type = info.client_info.devicetype.clone() as i8;
        let pubkey = info.tinc_info.pub_key.clone();
        std::mem::drop(info);

        Self {
            alias:                      None,
            appVersion:                 None,
            city:                       None,
            companyId:                  None,
            connectionLimit:            None,
            country:                    None,
            createBy:                   None,
            createTime:                 None,
            device_name:                 device_name.clone(),
            deviceSerial:               device_name,
            deviceType:                 device_type,
            hiddenFlag:                 false,
            id:                         None,
            ip:                         None,
            lan:                        None,
            latitude:                   None,
            longitude:                  None,
            pubKey:                     pubkey,
            region:                     None,
            tincStatus:                 0,
            updateBy:                   None,
            updateTime:                 None,
            userId:                     None,
            wan:                        None,
        }
    }

    pub fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}