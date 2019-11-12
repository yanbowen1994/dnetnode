use std::net::IpAddr;
extern crate uuid;

#[derive(Debug, Clone)]
pub struct ProxyInfo {
    pub uid:            String,
    pub ip:             Option<IpAddr>,
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
        #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
        let os = "arm-linux";
        #[cfg(all(not(target_arch = "arm"), target_os = "linux"))]
        let os = "linux";

        ProxyInfo {
            uid:            uuid::Uuid::new_v4().to_string(),
            ip:             None,
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