use std::path::PathBuf;

use super::parse_file::FileSettings;
use tinc_plugin::TincRunMode;
use crate::settings::default_settings::{DEFAULT_LINUX_DEFAULT_HOME_PATH, DEFAULT_PROXY_LOCAL_SERVER_PORT, DEFAULT_PROXY_TYPE, DEFAULT_LOG_LEVEL, DEFAULT_CLIENT_AUTO_CONNECT};

use super::error::*;

static mut EL: *mut Settings = 0 as *mut _;

#[derive(Clone, Debug, Deserialize)]
pub struct Common {
    pub conductor_url:   String,
    pub home_path:       PathBuf,
    pub log_level:       String,
    pub log_dir:         PathBuf,
    pub mode:            TincRunMode,
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

    fn default_running_mode() -> TincRunMode {
        TincRunMode::Client
    }

    fn default_log_level() -> String {
        DEFAULT_LOG_LEVEL.to_owned()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Proxy {
    pub local_port:                             String,
    pub local_https_server_certificate_file:    String,
    pub local_https_server_privkey_file:        String,
    pub proxy_type:                             String,
}

impl Proxy {
    fn empty() -> Self {
        Proxy {
            local_port:                            String::new(),
            local_https_server_certificate_file:   String::new(),
            local_https_server_privkey_file:       String::new(),
            proxy_type:                            String::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub common:         Common,
    pub proxy:          Proxy,
    pub client:         Client,
    pub last_runtime:   String,
}

impl Settings {
    pub fn new(config_dir: &str) -> Result<()> {
        let mut settings = FileSettings::load_config(config_dir)
            .and_then(|file_seting| {
                Self::parse_file_settings(file_seting)
            })
            .unwrap_or(Settings::default()?);

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
        Ok(Self {
            common,
            proxy,
            client,
            last_runtime: String::new(),
        })
    }

    fn parse_file_settings(mut file_settings: FileSettings) -> Result<Self> {
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
                            return TincRunMode::Proxy;
                        }
                        else if mode_str.to_lowercase() != "client" {
                            warn!("Invailed running mode setting. Proxy or Client.")
                        }
                        return TincRunMode::Client;
                    })
                    .unwrap_or(Common::default_running_mode());

                let username = file_common.username.unwrap_or("".to_owned());
                let password = file_common.password.unwrap_or("".to_owned());

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
            if common.mode == TincRunMode::Proxy {
                file_settings.proxy
                    .ok_or(Error::NoneError)
                    .and_then(|file_proxy| {
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
            if common.mode == TincRunMode::Client {
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

        Ok(Self {
            common,
            proxy,
            client,
            last_runtime: String::new(),
        })
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