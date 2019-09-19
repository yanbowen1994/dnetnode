use std::sync::{Arc, Mutex};
use std::str::FromStr;

use crate::info::{Info, get_info, get_mut_info};

use super::error::{Error, Result};
use crate::settings::get_settings;
use mac_address::get_mac_address;
use super::post;
use super::types::DeviceId;
use std::net::IpAddr;
use crate::info::team::{Team, TeamMember, DeviceProxy};

pub(super) fn search_team_by_mac() -> Result<()> {
    let url    = get_settings().common.conductor_url.clone()
        + "/vppn/api/v2/client/bindingdevice";;
    let device_id;
    {
        device_id = get_info().lock().unwrap().client_info.uid.clone();
    }

    let data = DeviceId {
        deviceid: device_id
    }.to_json();

    let mut res = post(&url, &data, "")?;

    if res.status().as_u16() == 200 {
        let res_data = &res.text().map_err(Error::Reqwest)?;
        let recv: JavaResponseTeamSearch = serde_json::from_str(res_data)
            .map_err(|e|{
                error!("search_team_by_mac - response data: {}", res_data);
                Error::ParseJsonStr(e)
            })?;

        if recv.code == Some(200) {
            let local_pubkey = get_info().lock().unwrap().client_info.pub_key.clone();

            let mut info = get_mut_info().lock().unwrap();
            let mut teams = recv.data;
            info.teams = teams
                .iter_mut()
                .map(|jteam|jteam.clone().into())
                .collect();

            for team in teams {
                let members = team.members;
                for member in members {
                    if member.pubkey == Some(local_pubkey.clone()) {
                        if let Some(vip) = &member.ip {
                            let vip = IpAddr::from_str(vip)
                                .map_err(|e|{
                                    error!("search_team_by_mac can't parse self vip.");
                                    Error::ParseIp(e)
                                })?;
                            info.client_info.vip = vip;
                        }
                    }
                }
            }
            return Ok(());
        }
        else {
            if let Some(msg) = recv.msg {
                return Err(Error::GetOnlineProxy(msg));
            }
        }
    }
    else {
        let mut err_msg = "Unknown reason.".to_string();
        if let Ok(msg) = res.text() {
            err_msg = msg;
        }
        return Err(Error::GetOnlineProxy(
            format!("Code:{} Msg:{}", res.status().as_u16(), err_msg).to_string()));
    }
    return Err(Error::GetOnlineProxy("Unknown reason.".to_string()));
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JavaResponseTeamSearch {
    pub code:               Option<u32>,
    pub data:               Vec<JavaResponseTeam>,
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


