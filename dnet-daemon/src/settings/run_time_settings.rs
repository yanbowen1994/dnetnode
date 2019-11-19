use std::path::PathBuf;
use std::str::FromStr;

use dnet_types::settings::{
    Settings as TypeSettings,
    Common as TypeCommon,
    Client as TypeClient,
    Proxy as TypeProxy,
    RunMode
};

use super::parse_file::FileSettings;
use super::default_settings::{DEFAULT_LINUX_DEFAULT_HOME_PATH, DEFAULT_PROXY_LOCAL_SERVER_PORT, DEFAULT_PROXY_TYPE, DEFAULT_LOG_LEVEL, DEFAULT_CLIENT_AUTO_CONNECT};
use super::error::*;
use std::net::IpAddr;
use crate::settings::error::Error::Config;

static mut EL: *mut Settings = 0 as *mut _;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Common {
    pub conductor_url:   String,
    pub home_path:       PathBuf,
    pub log_level:       String,
    pub log_dir:         PathBuf,
    pub mode:            RunMode,
    pub username:        String,
    pub password:        String,
}

impl Common {
    fn default() -> Result<Self> {
        let conductor_url = "".to_owned();
        let home_path = Self::default_home_path()?;
        let log_level = Self::default_log_level();
        let log_dir = Self::default_home_path()?;
        let mode = Self::default_running_mode();
        let username = "".to_owned();
        let password = "".to_owned();
        Ok(Self {
            conductor_url,
            home_path,
            log_level,
            log_dir,
            mode,
            username,
            password,
        })
    }

    fn default_home_path() -> Result<PathBuf> {
        dnet_path::home_dir(Some(DEFAULT_LINUX_DEFAULT_HOME_PATH))
            .ok_or(Error::home_path_not_set)
    }

    fn default_log_dir() -> Result<PathBuf> {
        Self::default_home_path().map(|home_dir|home_dir.join("log"))
    }

    fn default_running_mode() -> RunMode {
        RunMode::Client
    }

    fn default_log_level() -> String {
        DEFAULT_LOG_LEVEL.to_owned()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proxy {
    pub local_ip:                               Option<IpAddr>,
    pub local_port:                             String,
    pub local_https_server_certificate_file:    String,
    pub local_https_server_privkey_file:        String,
    pub proxy_type:                             String,
}

impl Proxy {
    fn empty() -> Self {
        Proxy {
            local_ip:                              None,
            local_port:                            String::new(),
            local_https_server_certificate_file:   String::new(),
            local_https_server_privkey_file:       String::new(),
            proxy_type:                            String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Client {
    pub auto_connect:                              bool,
}
impl Client {
    fn default() -> Self {
        Client {
            auto_connect: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tinc {
    pub tinc_memory_limit:                         f64,
    pub tinc_check_frequency:                      u32,
    pub tinc_allowed_out_memory_times:             u32,
    pub tinc_allowed_tcp_failed_times:             u32,
    pub tinc_debug_level:                          u32,
}
impl Tinc {
    fn default() -> Self {
        Tinc {
            tinc_memory_limit:                     100 as f64,
            tinc_check_frequency:                  0,
            tinc_allowed_out_memory_times:         0,
            tinc_allowed_tcp_failed_times:         0,
            tinc_debug_level:                      0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub common:         Common,
    pub proxy:          Proxy,
    pub client:         Client,
    pub tinc:           Tinc,
    pub last_runtime:   String,
}

impl Settings {
    pub fn new(config_dir: &str) -> Result<()> {
        let mut settings = FileSettings::load_config(config_dir)
            .and_then(|file_seting| {
                Self::parse_file_settings(file_seting)
            })?;

        let now = chrono::Utc::now().to_string();
        settings.last_runtime = now;

        unsafe {
            EL = Box::into_raw(Box::new(settings));
        };
        Ok(())
    }

    fn default() -> Result<Self> {
        let common = Common::default()?;
        let proxy = Proxy::empty();
        let client = Client::default();
        let tinc = Tinc::default();
        Ok(Self {
            common,
            proxy,
            client,
            tinc,
            last_runtime: String::new(),
        })
    }

    fn parse_file_settings(file_settings: FileSettings) -> Result<Self> {
        let common = file_settings.common
            .ok_or(Error::NoneError)
            .and_then(|file_common| {
                let conductor_url = file_common.conductor_url
                    .ok_or(Error::conductor_url_not_set)?;

                let home_path = file_common.home_path
                    .map(|home_path_str| PathBuf::from(home_path_str))
                    .unwrap_or(Common::default_home_path()?);

                let log_level = file_common.log_level.unwrap_or(Common::default_log_level());

                let log_dir = file_common.log_dir
                    .map(|log_dir_str|PathBuf::from(log_dir_str))
                    .unwrap_or(Common::default_log_dir()?);

                let mode = file_common.mode
                    .map(|mode_str| {
                        if mode_str.to_lowercase() == "proxy" {
                            return RunMode::Proxy;
                        }
                        else if mode_str.to_lowercase() != "client" {
                            warn!("Invailed running mode setting. Proxy or Client.")
                        }
                        return RunMode::Client;
                    })
                    .unwrap_or(Common::default_running_mode());
                let username;
                let password;
                #[cfg(target_arch = "arm")]
                    {
                        username = router_plugin::get_sn().ok_or(Error::GetSN)?;
                        password = username.clone();
                    }

                // If run router debug in x86, router_sn should setting in settings.toml.
                #[cfg(any(not(target_arch = "arm"), feature = "router_debug"))]
                    {
                        username = file_common.username.unwrap_or("".to_owned());
                        password = file_common.password.unwrap_or("".to_owned());
                    }

                Ok(Common {
                    conductor_url,
                    home_path,
                    log_level,
                    log_dir,
                    mode,
                    username,
                    password,
                })
        })
            .unwrap_or(Common::default()?);

        let proxy = {
            if common.mode == RunMode::Proxy {
                file_settings.proxy
                    .ok_or(Error::NoneError)
                    .and_then(|file_proxy| {
                        let local_ip = match file_proxy.local_ip {
                            Some(ip_str) => {
                                let ip = IpAddr::from_str(&ip_str)
                                    .map_err(|_|Error::Config("local_ip".to_string()))?;
                                Some(ip)
                            },
                            None => None,
                        };

                        let local_port = file_proxy.local_port
                            .unwrap_or(DEFAULT_PROXY_LOCAL_SERVER_PORT.to_owned());

                        let local_https_server_privkey_file = file_proxy.local_https_server_privkey_file
                            .ok_or(
                                Error::https_server_privkey_not_found
                            )?;

                        let local_https_server_certificate_file = file_proxy.local_https_server_certificate_file
                            .ok_or(
                                Error::https_server_certificate_not_found
                            )?;

                        let proxy_type = file_proxy.proxy_type.unwrap_or(
                            DEFAULT_PROXY_TYPE.to_owned()
                        );

                        Ok(Proxy {
                            local_ip,
                            local_port,
                            local_https_server_privkey_file,
                            local_https_server_certificate_file,
                            proxy_type,
                        })
                })?
            } else {
                Proxy::empty()
            }
        };

        let client = {
            if common.mode == RunMode::Client {
                file_settings.client.map(|file_client| {
                    let auto_connect = file_client.auto_connect
                        .map(|file_auto_connect|{
                            if file_auto_connect.to_lowercase() == "true" {
                                true
                            }
                            else {
                                false
                            }
                        })
                        .unwrap_or(DEFAULT_CLIENT_AUTO_CONNECT.to_owned());

                    Client {
                        auto_connect,
                    }
                })
                    .unwrap_or(Client::default())
            }
            else {
                Client::default()
            }
        };

        let tinc = file_settings.tinc
            .and_then(|file_settings| {
                let tinc_memory_limit = file_settings.tinc_memory_limit
                    .and_then(|x|x.parse::<f64>().ok())
                    .unwrap_or(100 as f64);
                let tinc_check_frequency = file_settings.tinc_check_frequency
                    .and_then(|x|x.parse::<u32>().ok())
                    .unwrap_or(0 as u32);
                let tinc_allowed_out_memory_times = file_settings.tinc_allowed_out_memory_times
                    .and_then(|x|x.parse::<u32>().ok())
                    .unwrap_or(0 as u32);
                let tinc_allowed_tcp_failed_times = file_settings.tinc_allowed_tcp_failed_times
                    .and_then(|x|x.parse::<u32>().ok())
                    .unwrap_or(0 as u32);
                let tinc_debug_level = file_settings.tinc_debug_level
                    .and_then(|x|x.parse::<u32>().ok())
                    .unwrap_or(0 as u32);

                Some(Tinc {
                    tinc_memory_limit,
                    tinc_check_frequency,
                    tinc_allowed_out_memory_times,
                    tinc_allowed_tcp_failed_times,
                    tinc_debug_level,
                })
            })
            .unwrap_or(Tinc::default());

        Ok(Self {
            common,
            proxy,
            client,
            tinc,
            last_runtime: String::new(),
        })
    }
}

impl Into<TypeSettings> for Settings {
    fn into(self) -> TypeSettings {
        TypeSettings {
            common: TypeCommon {
                conductor_url: self.common.conductor_url,
                home_path: self.common.home_path,
                log_level: self.common.log_level,
                log_dir: self.common.log_dir,
                mode: self.common.mode,
                username: self.common.username,
                password: self.common.password,
            },
            client: TypeClient {
                auto_connect: self.client.auto_connect,
            },
            proxy: TypeProxy {
                local_port: self.proxy.local_port,
                local_https_server_certificate_file: self.proxy.local_https_server_certificate_file,
                local_https_server_privkey_file: self.proxy.local_https_server_privkey_file,
                proxy_type: self.proxy.proxy_type,
            },
            last_runtime: self.last_runtime,
        }
    }
}

pub fn get_settings() -> &'static Settings {
    unsafe {
        if EL == 0 as *mut _ {
            panic!("Get settings instance, before init");
        }
        & *EL
    }
}

pub fn get_mut_settings() ->  &'static mut Settings {
    unsafe {
        if EL == 0 as *mut _ {
            panic!("Get settings instance, before init");
        }
        &mut *EL
    }
}

#[test]
fn test_setting() {
    Settings::new("./")
        .map_err(|e|{
            eprintln!("{:?}\n{}", e, e);
        })
        .expect("Error: Can not parse settings.");
    let _settings = get_settings();
}