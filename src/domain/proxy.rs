extern crate uuid;

use net_tool::{url_get, get_local_ip, get_wan_name};

use super::{TincInfo, GeoInfo};

#[derive(Debug, Clone)]
pub struct ProxyInfo {
    pub uid: String,
    pub proxy_pub_key: String,
    pub isregister: bool,
    pub cookie: String,
}
impl ProxyInfo {
    pub fn new() -> Self {
        ProxyInfo {
            uid: uuid::Uuid::new_v4().to_string(),
            proxy_pub_key: String::new(),
            isregister: false,
            cookie: String::new(),
        }
    }
}
