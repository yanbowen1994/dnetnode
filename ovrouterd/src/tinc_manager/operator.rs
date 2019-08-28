#![allow(dead_code)]

#[cfg(unix)]
use std::net::IpAddr;
use std::str::FromStr;
use std::fs;
use std::io::Write;

use domain::{Info, AuthInfo};
use settings::get_settings;
use tinc_plugin::{TincRunMode, ConnectTo};

use tinc_plugin::{TincOperator as PluginTincOperator,
                  TincInfo as PluginTincInfo,
                  TincOperatorError};

pub type Result<T> = std::result::Result<T, TincOperatorError>;

const TINC_AUTH_PATH: &str = "auth/";
const TINC_AUTH_FILENAME: &str = "auth.txt";

/// Tinc operator
pub struct TincOperator {
    tinc_handle: Option<duct::Handle>,
}
impl TincOperator {
    /// 获取tinc home dir 创建tinc操作。
    pub fn new() -> Self {
        let settings = get_settings();
        PluginTincOperator::new(&settings.tinc.home_path,
                                TincRunMode::Proxy);

        Self{
            tinc_handle: None,
        }
    }

    pub fn get_tinc_handle(&self) -> Option<&duct::Handle> {
        if let Some(child) = &self.tinc_handle {
            return Some(child);
        }
        None
    }

    /// 启动tinc 返回duct::handle
    pub fn start_tinc(&mut self) -> Result<()> {
        let tinc_handle = PluginTincOperator::instance().start_tinc()?;
        self.tinc_handle = Some(tinc_handle);
        return Ok(());
    }

    pub fn stop_tinc(&mut self) -> Result<()> {
        PluginTincOperator::instance().stop_tinc(&self.tinc_handle)
    }

    pub fn create_tinc_dirs(&self) -> Result<()> {
        PluginTincOperator::instance().create_tinc_dirs()
    }

    pub fn check_tinc_status(&mut self) -> Result<()> {
        PluginTincOperator::instance().check_tinc_status(&self.tinc_handle)
    }

    pub fn restart_tinc(&mut self) -> Result<()> {
        let tinc_handle = PluginTincOperator::instance()
            .restart_tinc(&self.tinc_handle)?;
        self.tinc_handle = Some(tinc_handle);
        return Ok(());
    }

    /// 添加子设备
    pub fn add_hosts(&self, host_name: &str, pub_key: &str) -> Result<()> {
        PluginTincOperator::instance().add_hosts(host_name, pub_key)
    }

    /// 获取子设备公钥
    pub fn get_host_pub_key(&self, host_name:&str) -> Result<String> {
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
    pub fn set_pub_key(&mut self, pub_key: &str) -> Result<()> {
        let path = self.tinc_home.clone() + &self.pub_key_path;
        let mut file =  fs::File::create(path.clone())
            .map_err(|_|Error::CreatePubKeyError)?;
        file.write(pub_key.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        return Ok(());
    }

    /// 获取本地tinc虚拟ip
    pub fn get_vip(&self) -> Result<IpAddr> {
        PluginTincOperator::instance().get_local_vip()
    }

    pub fn set_info_to_local(&self, info: &mut Info) -> Result<()> {
        let ip = IpAddr::from_str(&info.proxy_info.proxy_ip)
            .map_err(TincOperatorError::ParseLocalIpError)?;
        let vip = info.tinc_info.vip.to_owned();
        let pub_key = info.tinc_info.pub_key.to_owned();
        let mode = TincRunMode::Proxy;
        let connect_to: Vec<ConnectTo> = vec![];
        let tinc_info = PluginTincInfo {
            ip,
            vip,
            pub_key,
            mode,
            connect_to,
        };
        PluginTincOperator::instance().set_info_to_local(&tinc_info)
    }

    pub fn get_client_filename_by_virtual_ip(&self, vip: &str) -> String {
        PluginTincOperator::get_filename_by_ip(false, vip)
    }


    // 写TINC_AUTH_PATH/TINC_AUTH_FILENAME(auth/auth.txt),用于tinc reporter C程序
    // TODO 去除C上报tinc上线信息流程,以及去掉auth/auth.txt.
    pub fn write_auth_file(&self,
                           server_url:  &str,
                           info:        &Info,
    ) -> Result<()> {
        let settings = get_settings();
        let path = settings.tinc.home_path.to_string() + TINC_AUTH_PATH;
        let auth_dir = std::path::PathBuf::from(&(path));
        if !std::path::Path::new(&auth_dir).is_dir() {
            fs::create_dir_all(&auth_dir)
                .map_err(|e|TincOperatorError::IoError(path.clone() + " " + &e.to_string()))?;
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
                .map_err(|e|TincOperatorError::IoError(path.clone() + " " + &e.to_string()))?;
            let auth_info = AuthInfo::load(server_url, info);
            file.write(auth_info.to_json_str().as_bytes())
                .map_err(|e|TincOperatorError::IoError(path.clone() + " " + &e.to_string()))?;
        }

        return Ok(());
    }

    /// Load local tinc config file vpnserver for tinc vip and pub_key.
    /// Success return true.
    pub fn load_local(&mut self, tinc_home: &str, pub_key_path: &str) -> io::Result<TincInfo> {
        let mut tinc_info = TincInfo::new();
        {
            let mut res = String::new();
            let mut _file = fs::File::open(tinc_home.to_string() + pub_key_path)?;
            _file.read_to_string(&mut res)?;
            tinc_info.pub_key = res.clone();
        }
        {
            if let Ok(vip_str) = self.get_vip() {
                if let Ok(vip) = IpAddr::from_str(&vip_str) {
                    tinc_info.vip = vip;
                    return Ok(tinc_info);
                }

            }
        }
        return Err(io::Error::new(io::ErrorKind::InvalidData, "tinc config file error"));
    }

    fn set_tinc_up(&self) -> Result<()> {
        #[cfg(windows)]
            let buf = &(self.tinc_home.to_string() + "/tinc-report.exe -u");
        #[cfg(unix)]
            let buf = "#!/bin/sh\n".to_string() + &self.tinc_home + "/tinc-report -u";

        let path = self.tinc_home.clone() + "/" + TINC_UP_FILENAME;
        let mut file = fs::File::create(path.clone())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        file.write(buf.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        Ok(())
    }

    fn set_tinc_down(&self) -> Result<()> {
        #[cfg(windows)]
            let buf = &(self.tinc_home.to_string() + "/tinc-report.exe -d");
        #[cfg(unix)]
            let buf = "#!/bin/sh\n".to_string() + &self.tinc_home + "/tinc-report -d";

        let path = self.tinc_home.clone() + "/" + TINC_DOWN_FILENAME;
        let mut file = fs::File::create(path.clone())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        file.write(buf.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        Ok(())
    }

    fn set_host_up(&self) -> Result<()> {
        #[cfg(windows)]
            let buf = &(self.tinc_home.to_string() + "/tinc-report.exe -hu ${NODE}");
        #[cfg(unix)]
            let buf = "#!/bin/sh\n".to_string() + &self.tinc_home + "/tinc-report -hu ${NODE}";

        let path = self.tinc_home.clone() + "/" + HOST_UP_FILENAME;
        let mut file = fs::File::create(path.clone())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        file.write(buf.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        Ok(())
    }

    fn set_host_down(&self) -> Result<()> {
        #[cfg(windows)]
            let buf = &(self.tinc_home.to_string() + "/tinc-report.exe -hd ${NODE}");
        #[cfg(unix)]
            let buf = "#!/bin/sh\n".to_string() + &self.tinc_home + "/tinc-report -hd ${NODE}";

        let path = self.tinc_home.clone() + "/" + HOST_UP_FILENAME;
        let mut file = fs::File::create(path.clone())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        file.write(buf.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        Ok(())
    }
}

/// 获取出口网卡, 网卡名
pub fn get_wan_name() -> Result<String> {
    let output = duct::cmd(OsString::from("/bin/ip"),
                           vec!(OsString::from("route")))
        .read()
        .map_err(|e|Error::FailedToRunIp(e))?;
    match output
        .lines()
        .find(|line|line.trim().starts_with("default via "))
        .and_then(|line| line.trim().split_whitespace().nth(4)) {
        Some(dev) => Ok(dev.to_string()),
        None => Err(Error::NoWanDev)
    }
}
