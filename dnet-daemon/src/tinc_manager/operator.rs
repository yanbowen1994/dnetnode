#![allow(dead_code)]

use std::net::IpAddr;
use std::fs;
use std::io::Write;
use std::str::FromStr;

use tinc_plugin::{TincRunMode,
                  TincOperator as PluginTincOperator,
                  TincOperatorError};
use dnet_types::settings::RunMode;

use crate::info::{AuthInfo, get_info};
use crate::settings::get_settings;
use crate::settings::default_settings::TINC_INTERFACE;
use dnet_types::team::NetSegment;

pub type Result<T> = std::result::Result<T, TincOperatorError>;

const TINC_AUTH_PATH: &str = "auth/";
const TINC_AUTH_FILENAME: &str = "auth.txt";

/// Tinc operator
pub struct TincOperator {}
impl TincOperator {
    /// 获取tinc home dir 创建tinc操作。
    pub fn new() -> Self {
        if !PluginTincOperator::is_inited() {
            let tinc_home;
            let tinc_run_model;
            {
                let settings = get_settings();
                tinc_home = settings.common.home_path.clone();
                tinc_run_model = match &settings.common.mode {
                    RunMode::Proxy => TincRunMode::Proxy,
                    RunMode::Client => TincRunMode::Client,
                }
            }

            PluginTincOperator::new(&(tinc_home
                .join("tinc").to_str().unwrap().to_string() + "/"),
                                    tinc_run_model);
        }
        Self {}
    }

    pub fn init(&self) -> Result<()> {
        self.create_tinc_dirs()?;
        if !self.check_pub_key() {
            self.create_pub_key()?;
        }
        Ok(())
    }

    /// 启动tinc 返回duct::handle
    pub fn start_tinc(&mut self) -> Result<()> {
        self.set_info_to_local()?;
        PluginTincOperator::mut_instance().start_tinc()?;
        return Ok(());
    }

    pub fn set_routing(&self) -> Result<()> {
        let info = get_info().lock().unwrap();
        let local_vip = info.tinc_info.vip.ok_or(TincOperatorError::local_vip_not_init)?;

        let mut members_vip = vec![];
        for running_team in &info.client_info.running_teams {
            for member in &running_team.members {
                if &member.vip == &local_vip {
                    continue;
                }
                members_vip.push(member.vip.clone());
            }
        }

        std::mem::drop(info);

        let routing_table = sandbox::route::parse_routing_table();
        let routing_table = routing_table
            .iter()
            .filter_map(|route_info| {
                IpAddr::from_str(&route_info.dev).ok()
            })
            .collect::<Vec<IpAddr>>();

        for member_vip in members_vip {
            if !routing_table.contains(&member_vip) {
                sandbox::route::add_route(&member_vip, 32, TINC_INTERFACE);
                info!("routing table add {:?}", member_vip);
            }
        }

        #[cfg(any(target_arch = "arm", feature = "router_debug"))]
            {
                let info = get_info().lock().unwrap();
                let local_vip = info.client_info.device_info.lan.clone();

                let mut members_lan = vec![];
                for running_team in &info.client_info.running_teams {
                    for member in &running_team.members {
                        for member_lan in &member.lan {
                            if local_vip.contains(member_lan) {
                                continue;
                            }
                            else {
                                members_lan.push(member_lan.clone());
                            }
                        }


                    }
                }
                std::mem::drop(info);

                let routing_table = sandbox::route::parse_routing_table();
                let routing_table = routing_table
                    .iter()
                    .filter_map(|route_info| {
                        if let Ok(ip) = IpAddr::from_str(&route_info.dev) {
                            return Some(NetSegment {
                                ip,
                                mask: route_info.mask
                            });
                        }
                        None
                    })
                    .collect::<Vec<NetSegment>>();

                for member_lan in members_lan {
                    if !routing_table.contains(&member_lan) {
                        sandbox::route::add_route(&member_lan.ip, member_lan.mask, TINC_INTERFACE);
                        info!("routing table add {:?}/{}", member_lan.ip, member_lan.mask);
                    }
                }
            }

        Ok(())
    }

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
        let mut tinc = PluginTincOperator::mut_instance();
        tinc.check_tinc_status()?;
//        tinc.check_tinc_listen()?;
//        tinc.check_tinc_memory()?;
        Ok(())
    }

    pub fn restart_tinc(&mut self) -> Result<()> {
        PluginTincOperator::mut_instance().restart_tinc()
    }

    /// 添加子设备
    pub fn set_hosts(&self, is_proxy: bool, ip: &str, pubkey: &str) -> Result<()> {
        PluginTincOperator::instance().set_hosts(is_proxy, ip, pubkey)
    }

    /// 获取子设备公钥
    pub fn get_host_pub_key(&self, host_name: &str) -> Result<String> {
        PluginTincOperator::instance().get_host_pub_key(host_name)
    }

    /// openssl Rsa 创建2048位密钥对, 并存放到tinc配置文件中
    pub fn create_pub_key(&self) -> Result<()> {
        PluginTincOperator::instance().create_pub_key()
    }

    /// 从pub_key文件读取pub_key
    pub fn get_pub_key(&self) -> Result<String> {
        PluginTincOperator::instance().get_local_pub_key()
    }

    /// 修改本地公钥
//    pub fn set_pub_key(&mut self, pub_key: &str) -> Result<()> {
//        let path = self.tinc_home.clone() + &self.pub_key_path;
//        let mut file =  fs::File::create(path.clone())
//            .map_err(|_|Error::CreatePubKeyError)?;
//        file.write(pub_key.as_bytes())
//            .map_err(|e|TincOperatorError::IoError(path.clone() + " " + &e.to_string()))?;
//        return Ok(());
//    }

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
        PluginTincOperator::get_filename_by_ip(false, vip)
    }

    pub fn get_proxy_filename_by_virtual_ip(&self, vip: &str) -> String {
        PluginTincOperator::get_filename_by_ip(true, vip)
    }

    // 写TINC_AUTH_PATH/TINC_AUTH_FILENAME(auth/auth.txt),用于tinc reporter C程序
    // TODO 去除C上报tinc上线信息流程,以及去掉auth/auth.txt.
    pub fn write_auth_file(&self,
                           server_url: &str,
    ) -> Result<()> {
        let settings = get_settings();
        let path = settings.common.home_path.clone()
            .join("tinc").join("TINC_AUTH_PATH").to_str().unwrap().to_string();
        let auth_dir = std::path::PathBuf::from(&(path));
        if !std::path::Path::new(&auth_dir).is_dir() {
            fs::create_dir_all(&auth_dir)
                .map_err(|e| TincOperatorError::IoError(path.clone() + " " + &e.to_string()))?;
        }

        let file_path_buf = auth_dir.join(TINC_AUTH_FILENAME);
        let file_path = std::path::Path::new(&file_path_buf);

//        #[cfg(unix)]
//            {
//                let permissions = PermissionsExt::from_mode(0o644);
//                if file_path.is_file() {
//                    if let Ok(file) = fs::File::create(&file_path) {
//                        if let Err(_) = file.set_permissions(permissions) {
//                            ()
//                        }
//                    }
//                } else {
//                    let file = fs::File::create(&file_path)
//                        .map_err(|e| Error::FileCreateError(e.to_string()))?;
//                    let _ = file.set_permissions(permissions);
//                }
//            }
        if let Some(file_str) = file_path.to_str() {
            let path = file_str.to_string();
            let mut file = fs::File::create(path.clone())
                .map_err(|e| TincOperatorError::IoError(path.clone() + " " + &e.to_string()))?;
            let auth_info = AuthInfo::load(server_url);
            file.write(auth_info.to_json_str().as_bytes())
                .map_err(|e| TincOperatorError::IoError(path.clone() + " " + &e.to_string()))?;
        }

        return Ok(());
    }
}
