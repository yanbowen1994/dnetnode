#![allow(dead_code)]

use std::net::IpAddr;

use tinc_plugin::{TincRunMode, TincOperator as PluginTincOperator,
                  TincOperatorError, TincTools, TincSettings};
use dnet_types::settings::RunMode;

#[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
use dnet_types::team::NetSegment;

use crate::info::{get_info, get_mut_info};
use crate::settings::get_settings;

pub type Result<T> = std::result::Result<T, TincOperatorError>;

/// Tinc operator
pub struct TincOperator;
impl TincOperator {
    /// 获取tinc home dir 创建tinc操作。
    pub fn new() -> Self {
        if !PluginTincOperator::is_inited() {
            let settings = get_settings();
            let tinc_home = settings.common.home_path.clone()
                .join("tinc").to_str().unwrap().to_string() + "/";
            let tinc_run_model = match &settings.common.mode {
                RunMode::Proxy => TincRunMode::Proxy,
                RunMode::Client => TincRunMode::Client,
                RunMode::Center => TincRunMode::Center,
            };
            let port = settings.tinc.port;
            let tinc_settings = TincSettings {
                tinc_home,
                mode: tinc_run_model,
                port,
                tinc_memory_limit: settings.tinc.tinc_memory_limit,
                tinc_allowed_out_memory_times: settings.tinc.tinc_allowed_out_memory_times,
                tinc_allowed_tcp_failed_times: settings.tinc.tinc_allowed_tcp_failed_times,
                tinc_check_frequency: settings.tinc.tinc_check_frequency,
                external_boot: settings.tinc.external_boot,
            };

            PluginTincOperator::new(tinc_settings);
        }
        Self {}
    }

    pub fn init(&self) -> Result<()> {
        self.create_tinc_dirs()?;
        if !self.check_pub_key() {
            self.create_self_key_pair()?;
        }
        Ok(())
    }

    /// 启动tinc 返回duct::handle
    pub fn start_tinc(&mut self) -> Result<()> {
        self.set_info_to_local()?;
//        self.set_tinc_team_init_file()?;

        PluginTincOperator::mut_instance().start_tinc()?;
        let now = chrono::Utc::now().to_string();
        get_mut_info().lock().unwrap().tinc_info.last_runtime = Some(now);
        return Ok(());
    }

    fn set_tinc_team_init_file(&self) -> Result<()> {
        let run_mode = &get_settings().common.mode;

        if *run_mode == RunMode::Proxy || *run_mode == RunMode::Client {
            return Ok(());
        }

        let team_file = get_settings().common.home_path
            .join("tinc").join("tinc.group")
            .to_str().unwrap().to_string();
        let tinc_team = get_info().lock().unwrap()
            .teams.to_tinc_team();
        tinc_team.set_tinc_init_file(&team_file)
    }

//    pub fn set_routing(&self) -> Result<()> {
//        let info = get_info().lock().unwrap();
//        let local_vip = info.tinc_info.vip.ok_or(TincOperatorError::local_vip_not_init)?;
//
//        let mut members_vip = vec![];
//        for running_team_id in &info.teams.running_teams {
//            if let Some(running_team) = info.teams.all_teams.get(running_team_id) {
//                for member in &running_team.members {
//                    if member.vip == local_vip {
//                        continue;
//                    }
//                    members_vip.push(member.vip.clone());
//                }
//            }
//        }
//
//        std::mem::drop(info);
//
//        #[cfg(unix)]
//            {
//                let routing_table = sandbox::route::parse_routing_table()
//                    .ok_or(TincOperatorError::SetRoute)?;
//                let routing_table = routing_table
//                    .iter()
//                    .filter_map(|route_info| {
//                        IpAddr::from_str(&route_info.dev).ok()
//                    })
//                    .collect::<Vec<IpAddr>>();
//
//                for member_vip in members_vip {
//                    if !routing_table.contains(&member_vip) {
//                        sandbox::route::add_route(&member_vip,
//                                                  32,
//                                                  Some(TINC_INTERFACE.to_string()),
//                                                  None);
//                        info!("routing table add {:?}", member_vip);
//                    }
//                }
//            }
//
//        #[cfg(windows)]
//            {
//                for member_vip in members_vip {
//                    sandbox::route::add_route(&member_vip, 32, TINC_INTERFACE);
//                    info!("routing table add {:?}", member_vip);
//                }
//            }
//
//        #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
//            {
//                let info = get_info().lock().unwrap();
//                let local_vip = info.client_info.device_info.lan.clone();
//
//                let mut members_lan = vec![];
//                for running_team_id in &info.teams.running_teams {
//                    if let Some(running_team) = info.teams.all_teams.get(running_team_id) {
//                        for member in &running_team.members {
//                            for member_lan in &member.lan {
//                                if local_vip.contains(&member_lan) {
//                                    continue;
//                                } else {
//                                    members_lan.push(member_lan.clone());
//                                }
//                            }
//                        }
//                    }
//                }
//                std::mem::drop(info);
//
//                let routing_table = sandbox::route::parse_routing_table()
//                    .ok_or(TincOperatorError::SetRoute)?;
//                let routing_table = routing_table
//                    .iter()
//                    .filter_map(|route_info| {
//                        if let Ok(ip) = IpAddr::from_str(&route_info.dev) {
//                            return Some(NetSegment {
//                                ip,
//                                mask: route_info.mask
//                            });
//                        }
//                        None
//                    })
//                    .collect::<Vec<NetSegment>>();
//
//                for member_lan in members_lan {
//                    if !routing_table.contains(&member_lan) {
//                        sandbox::route::add_route(&member_lan.ip, member_lan.mask, TINC_INTERFACE);
//                        info!("routing table add {:?}/{}", member_lan.ip, member_lan.mask);
//                    }
//                }
//            }
//
//        Ok(())
//    }

    pub fn stop_tinc(&mut self) -> Result<()> {
        PluginTincOperator::mut_instance().stop_tinc()
    }

    pub fn create_tinc_dirs(&self) -> Result<()> {
        PluginTincOperator::instance().create_tinc_dirs()
    }

    pub fn check_pub_key(&self) -> bool {
        PluginTincOperator::instance().check_pub_key()
    }

    pub fn check_tinc_status(&mut self) -> Result<()> {
        let tinc = PluginTincOperator::mut_instance();
        tinc.check_tinc_status()?;
//        tinc.check_tinc_listen()?;
//        tinc.check_tinc_memory()?;
        Ok(())
    }

    pub fn get_tinc_connect_nodes(&self) -> Result<Vec<IpAddr>> {
        let tinc = PluginTincOperator::mut_instance();
        tinc.get_tinc_connect_nodes()
    }

    pub fn restart_tinc(&mut self) -> Result<()> {
        self.set_info_to_local()?;
//        self.set_tinc_team_init_file()?;
        PluginTincOperator::mut_instance().restart_tinc()?;
        let now = chrono::Utc::now().to_string();
        get_mut_info().lock().unwrap().tinc_info.last_runtime = Some(now);
        Ok(())
    }

    pub fn set_hosts(&self,
                     ip_port: Option<(IpAddr, u16)>,
                     vip:     IpAddr,
                     pubkey:  &str,
    ) -> Result<()> {
        PluginTincOperator::instance().set_hosts(
            ip_port,
            vip,
            pubkey,
        )
    }

    /// 获取子设备公钥
    pub fn get_host_pub_key(&self, host_name: &str) -> Result<String> {
        PluginTincOperator::instance().get_host_pub_key(host_name)
    }

    /// openssl Rsa 创建2048位密钥对, 并存放到tinc配置文件中
    pub fn create_self_key_pair(&self) -> Result<()> {
        PluginTincOperator::instance().create_self_key_pair()
    }

    /// 从pub_key文件读取pub_key
    pub fn get_pub_key(&self) -> Result<String> {
        PluginTincOperator::instance().get_local_pub_key()
    }

    /// 获取本地tinc虚拟ip
    pub fn get_vip(&self) -> Result<IpAddr> {
        PluginTincOperator::instance().get_local_vip()
    }

    pub fn set_info_to_local(&self) -> Result<()> {
        let tinc_info = get_info().lock().unwrap()
            .to_plugin_tinc_info()
            .map_err(|_|TincOperatorError::TincInfoError("Vip is None.".to_owned()))?;
        PluginTincOperator::mut_instance().set_info_to_local(&tinc_info)
    }

    pub fn get_client_filename_by_virtual_ip(&self, vip: &str) -> String {
        TincTools::get_filename_by_vip(false, vip)
    }

    pub fn get_proxy_filename_by_virtual_ip(&self, vip: &str) -> String {
        TincTools::get_filename_by_vip(true, vip)
    }
}
