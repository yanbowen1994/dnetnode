//! upload proxy status
#![allow(unreachable_code)]

use rustc_serialize::json;

use net_tool::url_post;
use domain::Info;

pub struct Client {
    url: String,
}
impl Client {
    pub fn new(url: String) -> Self {
        Client {
            url,
        }
    }
    
    pub fn proxy_register(&self, info: &Info) -> bool {
        let post = "vppn/api/v2/proxy/register";
        let url = self.url.to_string() + post;
        let data = Register::new_from_info(info).to_json();
        let (_res, _code) = url_post(&url, &data).unwrap_or(return false);
        if _code == 200 {
            return true
        }
        false
    }

    pub fn proxy_heart_beat(&self, info: &Info) -> bool {
        let post = "vppn/api/v2/proxy/hearBeat";
        let url = self.url.to_string() + post;
        let data = Heartbeat::new_from_info(info).to_json();
        let (_res, code) = url_post(&url, &data).unwrap_or(return false);
        if code == 200 {
            return true
        }
        false
    }

//    pub fn proxy_location(&self) -> bool {
//        let post = "/vppn/api/v2/proxy/upLocation";
//        let url = self.url.to_string() + post;
//        let (res, code) = url_post(&url, &data);
//        true
//    }
//
//    pub fn proxy_bind(&self) -> bool {
//        let post = "/vppn/api/v2/proxy/bindproxy";
//        let url = self.url.to_string() + post;
//        let (res, code) = url_post(&url, &data);
//        true
//    }
//
//    pub fn proxy_unbind(&self) -> bool {
//        let post = "/vppn/api/v2/proxy/unbindproxy";
//        let url = self.url.to_string() + post;
//        let (res, code) = url_post(&url, &data);
//        true
//    }
//
//    pub fn proxy_modify(&self) -> bool {
//        let post = "POST /vppn/api/v2/proxy/modifyproxy";
//        let url = self.url.to_string() + post;
//        let (res, code) = url_post(&url, &data);
//        true
//    }
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct Register {
    auth_id: String,
    auth_type: String,
    area: String,
    country_code: String,
    proxy_ip: String,
    pub_key: String,
    os: String,
    server_type: String,
    ssh_port: String,
    latitude: String,
    longitude: String,
}
impl Register {
    fn new_from_info(info :&Info) -> Self {
        Register {
            auth_id: info.proxy_info.uid.to_string(),
            auth_type: info.proxy_info.auth_type.to_string(),
            area: info.geo_info.area_code.to_string(),
            country_code: info.geo_info.country_code.to_string(),
            proxy_ip: info.proxy_info.proxy_ip.to_string(),
            pub_key: info.tinc_info.pub_key.to_string(),
            os: info.proxy_info.os.to_string(),
            server_type: info.proxy_info.server_type.to_string(),
            ssh_port: info.proxy_info.ssh_port.to_string(),
            latitude: info.geo_info.latitude.to_string(),
            longitude: info.geo_info.longitude.to_string(),
        }
    }
    
    fn to_json(&self) -> String {
        return json::encode(self).unwrap();
    }
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct Heartbeat {
    auth_id: String,
    proxy_ip: String,
    pub_key: String,
}
impl Heartbeat {
    fn new_from_info(info :&Info) -> Self {
        Heartbeat {
            auth_id: info.proxy_info.uid.to_string(),
            proxy_ip: info.proxy_info.proxy_ip.to_string(),
            pub_key: info.tinc_info.pub_key.to_string(),
        }
    }

    fn to_json(&self) -> String {
        return json::encode(self).unwrap();
    }
}


