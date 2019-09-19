use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

extern crate uuid;

use mac_address::get_mac_address;
use crate::tinc_manager::TincOperator;

use crate::net_tool::get_local_ip;
use crate::traits::InfoTrait;
use super::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub uid:        String,
    pub pub_key:    String,
    pub cookie:     String,
    pub vip:        IpAddr,
//    pub ssh_port: String,
//    pub auth_type: String,
//    pub os: String,
//    pub server_type: String,
//    pub ip: String,
//    pub vip: IpAddr,
//    pub online_porxy: Vec<OnlineProxy>,
}
impl ClientInfo {
    pub fn new() -> Result<Self> {
        Ok(Self {
            uid:        Self::get_uid()?,
            cookie:     "0cde13b523sf9aa5a403dc9f5661344b91d77609f70952eb488f31641".to_owned(),
            pub_key:    String::new(),
            vip:        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
//            proxy_pub_key: String::new(),
//            ssh_port: String::new(),
//            isregister: false,
//            auth_type: String::new(),
//            os: String::new(),
//            server_type: String::new(),
//            proxy_ip: "0.0.0.0".to_string(),
//            online_porxy: Vec::new(),
        })
    }

    fn get_uid() -> Result<String> {
        let mut uid= String::new();
        #[cfg(not(target_arc = "arm"))]
            {
                let mac = match get_mac_address() {
                    Ok(Some(ma)) => Some(ma.to_string().replace(":", "")),
                    Ok(None) => None,
                    Err(e) => None,
                }
                    .ok_or(Error::GetMac)?;
                #[cfg(target_os = "linux")]
                    let uid_ = "linux/".to_owned() + &mac;
                #[cfg(target_os = "macos")]
                    let uid_ = "macos/".to_owned() + &mac;
                #[cfg(target_os = "windows")]
                    let uid_ = "windows/".to_owned() + &mac;
                uid = uid_;
            }
        #[cfg(target_arch = "arm")]
            {
                uid = router_plugin::get_sn();
            }
        Ok(uid)
    }

    fn get_pubkey() -> String {
        let tinc = TincOperator::new();
        tinc.get_pub_key()
            .expect("Client get tinc pub_key, before it create.")
    }
}