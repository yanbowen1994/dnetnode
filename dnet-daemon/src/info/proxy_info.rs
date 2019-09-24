use std::net::IpAddr;
use std::str::FromStr;

extern crate uuid;

#[derive(Debug, Clone)]
pub struct ProxyInfo {
    pub uid:            String,
    pub ip:             IpAddr,
    pub proxy_pub_key:  String,
    pub isregister:     bool,
    pub cookie:         String,
    pub auth_type:      String,
    pub os:             String,
    pub server_type:    String,
    pub ssh_port:       String,
}
impl ProxyInfo {
    pub fn new() -> Self {
        #[cfg(target_os = "macos")]
        let os = "macos";
        #[cfg(target_os = "windows")]
        let os = "windows";
        #[cfg(target_arch = "arm")]
        let os = "arm-linux";
        #[cfg(all(not(target_arch = "arm"), target_os = "linux"))]
        let os = "linux";

        ProxyInfo {
            uid:            uuid::Uuid::new_v4().to_string(),
            ip:             IpAddr::from([255, 255, 255, 255]),
            proxy_pub_key:  String::new(),
            isregister:     false,
            cookie:         String::new(),
            auth_type:      "0".to_owned(),
            os:             os.to_owned(),
            server_type:    "vppn1".to_owned(),
            ssh_port:       String::new(),
        }
    }
}