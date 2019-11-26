use std::path::PathBuf;
use std::net::IpAddr;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RunMode {
    Proxy,
    Client,
    Center,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Common {
    pub conductor_url:                          String,
    pub home_path:                              PathBuf,
    pub log_level:                              String,
    pub log_dir:                                PathBuf,
    pub mode:                                   RunMode,
    pub username:                               String,
    pub password:                               String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proxy {
    pub local_ip:                               Option<IpAddr>,
    pub local_port:                             u16,
    pub local_https_server_certificate_file:    String,
    pub local_https_server_privkey_file:        String,
    pub proxy_type:                             String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Client {
    pub auto_connect:                           bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub common:                                 Common,
    pub proxy:                                  Proxy,
    pub client:                                 Client,
    pub last_runtime:                           String,
}
