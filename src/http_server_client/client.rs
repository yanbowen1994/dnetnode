//! upload proxy status
use std::net::IpAddr;
use std::str::FromStr;

use rustc_serialize::json;

use net_tool::url_post;
use domain::Info;
use domain::OnlineProxy;
use settings::Settings;
use net_tool::http_json;

#[derive(Debug)]
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

        debug!("proxy_login - request url: {} ",url);
        debug!("proxy_login - request data:{}",data);

        let post = ||
            {
                loop {
                    let _res = match url_post(&url, data.clone(), "") {
                        Ok(x) => {
                            Some(x)
                        },
                        Err(e) => {
                            error!("proxy_login - response {}", e);
                            None
                        }
                    };
                    if let Some(x) = _res {
                        return Some(x);
                    }
                    else {
                        continue;
                    }
                }
                None
            };
        let res = match post() {
            Some(x) => x,
            None => {
                error!("proxy_register - response");
                return false;
            }
        };

        debug!("proxy_login - response code: {}",res.code);
        debug!("proxy_login - response header: {:?}",res.header);
        debug!("proxy_login - response data: {:?}",res.data);

        if res.code == 200 {
            let header = res.header;
            let cookie = header_cookie(header);
            info.proxy_info.cookie = cookie;

            let res_data = res.data;

            debug!("proxy_login - response cookie: {}",info.proxy_info.cookie);

            let _login: Login = match json::decode(&res_data) {
                Ok(login) => {
                    debug!("proxy_login resolve json result: {:?}",login);
                    login
                },
                Err(e) => {
                    error!("proxy_login resolve json exception: {:?}", e);
                    return false;
                }
            };
            return true;
        }
        return false;
    }
    
    pub fn proxy_register(&self, info: &mut Info) -> bool {
        let post = "/vppn/api/v2/proxy/register";
        let url = self.url.to_string() + post;
        let data = Register::new_from_info(info).to_json();
        let cookie = info.proxy_info.cookie.clone();
        debug!("proxy_register - request info: {:?}",info);
        debug!("proxy_register - request url: {}",url);
        debug!("proxy_register - request data: {}",data);
        let post = ||
            {
                loop {
                    let _res = match url_post(&url, data.clone(), &cookie) {
                        Ok(x) => {
                            Some(x)
                        },
                        Err(e) => {
                            error!("proxy_register - response {}", e);
                            None
                        }
                    };
                    if let Some(x) = _res {
                        return Some(x);
                    }
                    else {
                        continue;
                    }
                }
                None
            };
        let res = match post() {
            Some(x) => x,
            None => {
                error!("proxy_register - response");
                return false;
            }
        };

        debug!("proxy_register - response code: {}",res.code);
        debug!("proxy_register - response data: {:?}",res.data);

        if res.code == 200 {
            let recv: RegisterRecv = match json::decode(&res.data) {
                Ok(x) => x,
                Err(e) => {
                    error!("proxy_register - resolve json: {:?}", e);
                    return false;
                }
            };
            if recv.code == 200 {
                info.proxy_info.isregister = true;
                return true;
            }
        }
        false
    }

    pub fn proxy_get_online_proxy(&self, info: &mut Info) -> bool {
        let post = "/vppn/api/v2/proxy/getonlineproxy";
        let url = self.url.to_string() + post;
        let data = Register::new_from_info(info).to_json();
        let cookie = info.proxy_info.cookie.clone();
        trace!("proxy_get_online_proxy - request info: {:?}",info);
        debug!("proxy_get_online_proxy - request url: {}",url);
        trace!("proxy_get_online_proxy - request data: {}",data);

        let post = ||
            {
                loop {
                    let _res = match url_post(&url, data.clone(), &cookie) {
                        Ok(x) => {
                            Some(x)
                        },
                        Err(e) => {
                            error!("proxy_get_online_proxy - response {}", e);
                            None
                        }
                    };
                    if let Some(x) = _res {
                        return Some(x);
                    }
                    else {
                        continue;
                    }
                }
                None
            };
        let res = match post() {
            Some(x) => x,
            None => {
                error!("proxy_register - response");
                return false;
            }
        };

        debug!("proxy_get_online_proxy - response code: {}",res.code);

        if res.code == 200 {
            let recv: GetOnlinePorxyRecv = match json::decode(&res.data) {
                Ok(x) => x,
                Err(e) => {
                    error!("proxy_get_online_proxy - resolve json: {:?}", e);
                    return false;
                }
            };
            if recv.code == 200 {
                let proxy_vec: Vec<Proxy> = recv.data;
                let local_pub_key = info.tinc_info.pub_key.clone();
                let mut other_proxy = vec![];
                for proxy in proxy_vec {
                    if proxy.pubkey.to_string() == local_pub_key {
                        if let Ok(vip) = IpAddr::from_str(&proxy.vip) {
                            info.tinc_info.vip = vip;
                        }
                        else {
                            error!("proxy_get_online_proxy - get online proxy data invalid");
                            return false;
                        }
                    }
                    else {
                        if let Ok(other_ip) = IpAddr::from_str(&proxy.ip) {
                            if let Ok(other_vip) = IpAddr::from_str(&proxy.vip) {
                                let other = OnlineProxy::from(other_ip, other_vip, proxy.pubkey);
                                other_proxy.push(other);
                                continue
                            }
                        }
                        error!("proxy_get_online_proxy - get online proxy data invalid");
                    }
                }
                info.proxy_info.online_porxy = other_proxy;
                return true;
            }
        }
        false
    }

    pub fn proxy_heart_beat(&self, info: &Info) -> bool {
        let post = "/vppn/api/v2/proxy/hearBeat";
        let url = self.url.to_string() + post;
        let data = Heartbeat::new_from_info(info).to_json();
        let cookie = info.proxy_info.cookie.clone();

        debug!("proxy_heart_beat - request url: {}",url);
        debug!("proxy_heart_beat - request data: {}",data);

        let post = ||
            {
                loop {
                    let _res = match url_post(&url, data.clone(), &cookie) {
                        Ok(x) => {
                            Some(x)
                        },
                        Err(e) => {
                            error!("proxy_heart_beat - response {}", e);
                            None
                        }
                    };
                    if let Some(x) = _res {
                        return Some(x);
                    }
                    else {
                        continue;
                    }
                }
                None
            };
        let res = match post() {
            Some(x) => x,
            None => {
                error!("proxy_register - response");
                return false;
            }
        };

        debug!("proxy_heart_beat - response code: {}",res.code);
        debug!("proxy_heart_beat - response data: {:?}",res.data);

        if res.code == 200 {
            let recv: Recv = match json::decode(&res.data) {
                Ok(x) => x,
                Err(e) => {
                    error!("proxy_heart_beat - response: {:?}", e);
                    return false;
                }
            };
            if recv.code == 200 {
                return true;
            }
        }
        false
    }
}

fn header_cookie(header: Vec<String>) -> String {
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

#[allow(non_snake_case)]
#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct JavaRegister {
    authId: String,
    authType: String,
    area: String,
    countryCode: String,
    proxyIp: String,
    pubKey: String,
    os: String,
    serverType: String,
    sshPort: String,
    latitude: String,
    longitude: String,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct Register {
    auth_id: String,
    auth_type: String,
    area: String,
    countryCode: String,
    proxyIp: String,
    pubKey: String,
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
            countryCode: info.geo_info.country_code.to_string(),
            proxyIp: info.proxy_info.proxy_ip.to_string(),
            pubKey: info.tinc_info.pub_key.to_string(),
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

#[allow(non_snake_case)]
#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct VecProxy {
    inner: Vec<Proxy>,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct Proxy {
    id:                 String,
    ip:                 String,
    country:            String,
    region:             String,
    city:               String,
    username:           Option<String>,
    teamcount:          u32,
    ispublic:           bool,
    vip:                String,
    pubkey:             String,

}

#[allow(non_snake_case)]
#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct Heartbeat {
    authID: String,
    proxyIp: String,
    pubKey: String,
}
impl Heartbeat {
    fn new_from_info(info :&Info) -> Self {
        Heartbeat {
            authID:    info.proxy_info.uid.to_string(),
            proxyIp:   info.proxy_info.proxy_ip.to_string(),
            pubKey:    info.tinc_info.pub_key.to_string(),
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
    code:    i32,
    data:    LoginUser,
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct LoginUser {
    userid:                         String,
    username:                       String,
    useremail:                      String,
    photo:                          Option<String>,
    devices:                        Option<Vec<Device>>,
    enable_autogroup:               bool,
    enable_autoothergroup:          bool,
    enable_autonetworking:          bool,
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct Device {
    deviceid:    Option<String>,
    devicename:  Option<String>,
    devicetype:  Option<i32>,
    lan:         Option<String>,
    wan:         Option<String>,
    ip:          Option<String>,
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct Recv {
    code:        i32,
    msg:         Option<String>,
    data:        Option<String>,
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct RegisterRecv {
    code:        i32,
    msg:         Option<String>,
    data:        Option<JavaRegister>,
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
struct GetOnlinePorxyRecv {
    code:        i32,
    msg:         Option<String>,
    data:        Vec<Proxy>,
}