use std::net::{IpAddr, Ipv4Addr};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProxyInfo {
    pub auth_id:                    Option<String>,
    pub auth_type:                  Option<String>,
    pub city:                       Option<String>,
    pub company_id:                 Option<String>,
    pub connections:                u32,
    pub country:                    Option<String>,
    pub edges:                      u32,
    pub id:                         Option<String>,
    pub ip:                         Option<IpAddr>,
    pub latitude:                   Option<String>,
    pub longitude:                  Option<String>,
    pub nodes:                      u32,
    pub os:                         Option<String>,
    pub pubkey:                     String,
    pub public_flag:                bool,
    pub region:                     Option<String>,
    pub server_port:                u16,
    pub server_type:                Option<String>,
    pub ssh_port:                   Option<String>,
    pub status:                     i32,
    pub tinc_port:                  u16,
    pub user_id:                    Option<String>,
    pub vip:                        IpAddr,
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

        Self {
            auth_id:                    None,
            auth_type:                  None,
            city:                       None,
            company_id:                 None,
            connections:                0,
            country:                    None,
            edges:                      0,
            id:                         None,
            ip:                         None,
            latitude:                   None,
            longitude:                  None,
            nodes:                      0,
            os:                         Some(os.to_owned()),
            pubkey:                     String::new(),
            public_flag:                true,
            region:                     None,
            server_port:                0,
            server_type:                None,
            ssh_port:                   None,
            status:                     0,
            tinc_port:                  0,
            user_id:                    None,
            vip:                        IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)),
        }
    }
}