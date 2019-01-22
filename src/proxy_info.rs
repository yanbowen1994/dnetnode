use std::net::{Ipv4Addr, SocketAddr, TcpStream, IpAddr};

extern crate serde_json;
use self::serde_json::{Value, Error};

use net_tool::url_get;

#[derive(Debug, Clone)]
pub struct TincInfo {
    uid:String,
    proxy_ip:IpAddr,
    country:String,
    city:String,
    region:String,
    os:String,
    last_heartbeat:String,
    pub_key:String,
}

#[derive(Debug, Clone)]
pub struct ProxyInfo {
    pub geo_json:Value,
    pub tinc_info:TincInfo,
}
impl ProxyInfo {
    pub fn flush_geo_info(&mut self) {
        let (res, _) = url_get("http://52.25.79.82:10000/geoip_json.php");
        let json: Value = serde_json::from_str(&res).unwrap();
        self.geo_json = json;
    }

//    pub fn get_tinc_info(&self) -> Value {
//        self.tinc_info.clone()
//    }

    pub fn set_tinc_info(&mut self) {

    }
}