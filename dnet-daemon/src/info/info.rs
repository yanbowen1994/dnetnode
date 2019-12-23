use std::sync::Mutex;

use tinc_plugin::{TincInfo as PluginTincInfo, TincRunMode};
use dnet_types::settings::RunMode;
use dnet_types::proxy::ProxyInfo;
use dnet_types::team::Team;
use dnet_types::status::{Status, TunnelState};

use crate::settings::get_settings;
use super::error::{Error, Result};
use super::{TeamInfo, NodeInfo, UserInfo, ClientInfo, TincInfo};

static mut EL: *mut Mutex<Info> = 0 as *mut _;

#[derive(Debug, Clone)]
pub struct Info {
    pub client_info:        ClientInfo,
    pub proxy_info:         ProxyInfo,
    pub tinc_info:          TincInfo,
    pub status:             Status,
    pub teams:              TeamInfo,
    pub user:               UserInfo,
    pub node:               NodeInfo,
}

impl Info {
    pub fn new() -> Result<()> {
        let client_info = ClientInfo::new()?;
        let mut proxy_info = ProxyInfo::new();
        proxy_info.auth_id = Some(uuid::Uuid::new_v4().to_string());

        info!("local uid: {:?}", proxy_info.auth_id);

        let mut tinc_info = TincInfo::new();
        tinc_info.load_local();

        debug!("client_info: {:?}", client_info);
        debug!("proxy_info: {:?}", proxy_info);
        debug!("tinc_info: {:?}", tinc_info);

        let info = Info {
            client_info,
            proxy_info,
            tinc_info,
            status: Status::new(),
            teams:  TeamInfo::new(),
            user:   UserInfo::new(),
            node:   NodeInfo::new(),
        };

        unsafe {
            EL = Box::into_raw(Box::new(Mutex::new(info)));
        }

        Ok(())
    }

    pub fn to_plugin_tinc_info(&self) -> Result<PluginTincInfo> {
        let settings = get_settings();
        let tinc_run_model = match &settings.common.mode {
            RunMode::Proxy => TincRunMode::Proxy,
            RunMode::Client => TincRunMode::Client,
            RunMode::Center => TincRunMode::Center,
        };

        if let Some(vip) = &self.tinc_info.vip {
            return Ok(PluginTincInfo {
                ip:             self.proxy_info.ip,
                vip:            vip.clone(),
                port:           settings.tinc.port,
                pub_key:        self.tinc_info.pub_key.clone(),
                mode:           tinc_run_model,
                connect_to:     self.tinc_info.connect_to.clone(),
            })
        }
        return Err(Error::TincInfoVipNotFound);
    }

    pub fn flush_self_from_plugin_info(&mut self, new_info: &PluginTincInfo) -> Result<bool> {
        let old_info = self.to_plugin_tinc_info()?;
        if old_info != *new_info {
            self.tinc_info.vip = Some(new_info.vip.clone());
            self.tinc_info.pub_key = new_info.pub_key.clone();
            self.tinc_info.connect_to = new_info.connect_to.clone();
            self.proxy_info.ip = new_info.ip.clone();
            return Ok(true)
        }
        else {
            Ok(false)
        }
    }

    pub fn fresh_running_from_all(&mut self) {
        let self_name = &self.client_info.device_name;
        let mut running_teams = vec![];
        for (team_id, team) in &self.teams.all_teams {
            for member in &team.members {
                if let Some(device_name) = &member.device_name {
                    if device_name == self_name && member.connect_status == true {
                        running_teams.push(team_id.clone());
                        break
                    }
                }
            }
        }
        self.teams.running_teams = running_teams;
    }

    pub fn get_current_team_connect(&self) -> Vec<Team> {
        let mut teams = self.teams.all_teams
            .values()
            .cloned()
            .collect::<Vec<Team>>();
        let _ = teams.iter_mut()
            .map(|team| {
                let _ = team.members.iter_mut()
                    .map(|member| {
                        if self.tinc_info.current_connect.contains(&member.vip) {
                            member.is_local_tinc_host_up  = Some(true);
                        }
                        else if let Some(self_vip) = self.tinc_info.vip {
                            if self_vip == member.vip {
                                member.is_local_tinc_host_up  = Some(self.status.tunnel == TunnelState::Connected);
                            }
                        }
                    })
                    .collect::<Vec<()>>();
            })
            .collect::<Vec<()>>();

        teams
    }
}

pub fn get_info() -> &'static Mutex<Info> {
    unsafe {
        if EL == 0 as *mut _ {
            panic!("Get settings instance, before init");
        }
        & *EL
    }
}

pub fn get_mut_info() ->  &'static mut Mutex<Info> {
    unsafe {
        if EL == 0 as *mut _ {
            panic!("Get settings instance, before init");
        }
        &mut *EL
    }
}