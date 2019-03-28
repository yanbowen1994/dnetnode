use std::net::IpAddr;
use std::str::FromStr;

extern crate uuid;

use net_tool::{get_local_ip};

#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub struct OnlineProxy {
    pub ip:                 IpAddr,
    pub vip:                IpAddr,
    pub pubkey:             String,
}
impl OnlineProxy {
    pub fn new() -> Self {
        Self {
            ip:     IpAddr::from_str("255.255.255.255").unwrap(),
            vip:    IpAddr::from_str("255.255.255.255").unwrap(),
            pubkey: String::new(),
        }
    }
    pub fn from(ip:IpAddr, vip:IpAddr, pubkey:String) -> Self {
        Self {
            ip,
            vip,
            pubkey,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProxyInfo {
    pub uid: String,
    pub proxy_pub_key: String,
    pub isregister: bool,
    pub cookie: String,
    pub auth_type: String,
    pub os: String,
    pub server_type: String,
    pub proxy_ip: String,
    pub ssh_port: String,
    pub online_porxy: Vec<OnlineProxy>,
}
impl ProxyInfo {
    pub fn new() -> Self {
        ProxyInfo {
            uid: String::new(),
            proxy_pub_key: String::new(),
            isregister: false,
            cookie: String::new(),
            auth_type: String::new(),
            os: String::new(),
            server_type: String::new(),
            proxy_ip: "0.0.0.0".to_string(),
            ssh_port: String::new(),
            online_porxy: Vec::new(),
        }
    }

    pub fn create_uid(&mut self) -> bool {
        self.uid = uuid::Uuid::new_v4().to_string();
        true
    }

    pub fn load_local(&mut self) -> bool {
        self.auth_type = "0".to_string();
        self.server_type = "vppn1".to_string();
        self.os = "ubuntu".to_string();
        if let Ok(local_ip) = get_local_ip() {
            self.proxy_ip = local_ip.to_string();
        } else {
            return false;
        };
        true
    }
}