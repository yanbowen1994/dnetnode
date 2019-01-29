//! upload proxy status

extern crate httparse;
//use self::httparse::;
use rustc_serialize::json;

use net_tool::url_post;
use domain::Info;
use settings::Settings;
use net_tool::http_json;

pub struct Client {
    url: String,
}
impl Client {
    pub fn new(url: String) -> Self {
        Client {
            url,
        }
    }

    pub fn proxy_login(&self, settings: &Settings, info: &mut Info) -> bool {
        let post = "/login";
        let url = self.url.to_string() + post;
        let data = http_json(User::new_from_settings(settings).to_json());

        let res = match url_post(&url, data, "".to_string()) {
            Ok(res) => res,
            Err(e) => {
                println!("{:?}", e);
                return false;
            }
        };

        if res.code == 200 {
            let header = res.header;
            let cookie = header_cookie(header);
            info.proxy_info.cookie = cookie;

            let res_data = res.data;

            let login: Login = match json::decode(&res_data) {
                Ok(login) => login,
                Err(e) => {
                    println!("{:?}", e);
                    return false;
                }
            };
//            info.proxy_info.isregister = login.data.enable_autogroup
//            login.data.username

            return true;
        }
        return false;
    }
    
    pub fn proxy_register(&self, info: &Info) -> Option<String> {
        let post = "/vppn/api/v2/proxy/register";
        let url = self.url.to_string() + post;
        let data = Register::new_from_info(info).to_json();
        let res = match url_post(&url, data, info.proxy_info.cookie.clone()) {
            Ok(res) => res,
            Err(e) => {
                println!("{:?}", e);
                return None;
            }
        };
        if res.code == 200 {
            return Some(res.data);
        }
        None
    }

    pub fn proxy_heart_beat(&self, info: &Info) -> bool {
        let post = "/vppn/api/v2/proxy/hearBeat";
        let url = self.url.to_string() + post;
        let data = Heartbeat::new_from_info(info).to_json();
        let res = match url_post(&url, data, info.proxy_info.cookie.clone()) {
            Ok(res) => res,
            Err(e) => {
                println!("{:?}", e);
                return false;
            }
        };
        if res.code == 200 {
//            return Some(res.data);
            return true;
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

fn header_cookie(mut header: Vec<String>) -> String {
    let mut headers_str = String::new();
    for i in 0..header.len() {
        let slice = header[i].clone();
        if let Some(_) = slice.find("Set-Cookie") {
            headers_str += &slice.replace("Set-Cookie: ", "");
            break
        }
    }
    headers_str
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

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct User {
    username: String,
    password: String,
}
impl User {
    fn new_from_settings(settings: &Settings) -> Self {
        User {
            username: settings.client.username.clone(),
            password: settings.client.password.clone(),
        }
    }
    fn to_json(&self) -> String {
        return json::encode(self).unwrap();
    }
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct Login {
    code: i32,
    data: LoginUser,
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct LoginUser {
    userid:                         String,
    username:                       String,
    useremail:                      String,
    photo:                          String,
    devices:                        Vec<Device>,
    enable_autogroup:               bool,
    enable_autoothergroup:          bool,
    enable_autonetworking:          bool,
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct Device {
    deviceid:    String,
    devicename:  String,
    devicetype:  i32,
    lan:         String,
    wan:         String,
    ip:          String,
}