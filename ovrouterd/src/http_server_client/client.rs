//! upload proxy status
use std::net::IpAddr;
use std::str::FromStr;

use net_tool::url_post;
use domain::Info;
use domain::OnlineProxy;
use settings::Settings;
use std::time::{Instant, Duration};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Login can not parse json str.")]
    LoginParseJsonStr(#[error(cause)] serde_json::Error),

    #[error(display = "Login failed no cookie back.")]
    LoginResNoCookie,

    #[error(display = "Login failed.")]
    LoginFailed(String),

    #[error(display = "Proxy register failed.")]
    RegisterFailed(String),

    #[error(display = "Login can not parse json str.")]
    RegisterParseJsonStr(#[error(cause)] serde_json::Error),

    #[error(display = "Get online proxy failed.")]
    GetOnlineProxy(String),

    #[error(display = "Login can not parse json str.")]
    GetOnlineProxyParseJsonStr(#[error(cause)] serde_json::Error),

    #[error(display = "proxy_get_online_proxy - get online proxy data invalid.")]
    GetOnlineProxyInvalidData,

    #[error(display = "Heartbeat can not parse json str.")]
    HeartbeatJsonStr(#[error(cause)] serde_json::Error),

    #[error(display = "Heartbeat timeout.")]
    HeartbeatTimeout,

    #[error(display = "Heartbeat failed.")]
    HeartbeatFailed,

    #[error(display = "reqwest::Error.")]
    Reqwest(#[error(cause)] reqwest::Error),
}

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

    pub fn proxy_login(&self,
                       settings:    &Settings,
                       info:        &mut Info,
    ) -> Result<()> {
        let post = "/login";
        let url = self.url.to_string() + post;
        let data = User::new_from_settings(settings).to_json();

        debug!("proxy_login - request url: {} ",url);
        debug!("proxy_login - request data:{}",data);

        let post = || {
            loop {
                let _res = match url_post(&url, &data, "") {
                    Ok(x) => return x,
                    Err(e) => {
                        error!("proxy_login - response {}", e);
                        continue;
                    }
                };
            }
        };
        let mut res = post();

        debug!("proxy_login - response code: {}", res.status().as_u16());
        debug!("proxy_login - response header: {:?}", res.headers());

        if res.status().as_u16() == 200 {
            {
                let cookie = match res.cookies().next() {
                    Some(cookie) => cookie,
                    None => {
                        return Err(Error::LoginResNoCookie);
                    }
                };
                let cookie_str = cookie.value();
                let cookie_str = &("Set-Cookie=".to_string() + cookie_str);
                debug!("proxy_login - response cookie: {}", cookie_str);
                info.proxy_info.cookie = cookie_str.to_string();
            }

            let res_data = res.text().map_err(Error::Reqwest)?;
            debug!("proxy_login - response data: {:?}", res_data);
            let _login: Login = serde_json::from_str(&res_data)
                .map_err(Error::LoginParseJsonStr)?;

            return Ok(());
        }
        else {
            let mut err_msg = "Unknown reason.".to_string();
            if let Ok(msg) = res.text() {
                err_msg = msg;
            }
            return Err(Error::LoginFailed(
                format!("Code:{} Msg:{}", res.status().as_u16(), err_msg).to_string()));
        }
        return Err(Error::LoginFailed("Unknown reason.".to_string()));
    }

    pub fn proxy_register(&self,
                          info: &mut Info)
                          -> Result<()>  {
        let post = "/vppn/api/v2/proxy/register";
        let url = self.url.to_string() + post;
        let data = Register::new_from_info(info).to_json();
        let cookie = info.proxy_info.cookie.clone();
        debug!("proxy_register - request info: {:?}", info);
        debug!("proxy_register - request url: {}", url);
        debug!("proxy_register - request data: {}", data);
        let post = || {
            loop {
                let _res = match url_post(&url, &data, &cookie) {
                    Ok(x) => return x,
                    Err(e) => {
                        error!("proxy_register - response {}", e);
                        continue;
                    }
                };
            }
        };
        let mut res = post();

        debug!("proxy_register - response code: {}", res.status().as_u16());

        if res.status().as_u16() == 200 {
            let recv: RegisterRecv = serde_json::from_str(
                &res.text().map_err(Error::Reqwest)?
            ).map_err(Error::RegisterParseJsonStr)?;

            debug!("proxy_register - response data: {:?}", recv);

            if recv.code == 200 {
                info.proxy_info.isregister = true;
                return Ok(());
            }
            else {
                if let Some(msg) = recv.msg {
                    return Err(Error::RegisterFailed(msg));
                }
            }
        }
        else {
            let mut err_msg = "Unknown reason.".to_string();
            if let Ok(msg) = res.text() {
                err_msg = msg;
            }
            return Err(Error::RegisterFailed(
                format!("Code:{} Msg:{}", res.status().as_u16(), err_msg).to_string()));
        }
        Err(Error::RegisterFailed("Unknown reason.".to_string()))
    }

    pub fn proxy_get_online_proxy(&self, info: &mut Info) -> Result<()> {
        let post = "/vppn/api/v2/proxy/getonlineproxy";
        let url = self.url.to_string() + post;
        let data = Register::new_from_info(info).to_json();
        let cookie = info.proxy_info.cookie.clone();
        trace!("proxy_get_online_proxy - request info: {:?}",info);
        debug!("proxy_get_online_proxy - request url: {}",url);
        trace!("proxy_get_online_proxy - request data: {}",data);

        let post = || {
            loop {
                let _res = match url_post(&url, &data, &cookie) {
                    Ok(x) => {
                        return x;
                    },
                    Err(e) => {
                        error!("proxy_get_online_proxy - response {}", e);
                        continue;
                    }
                };
            }
        };
        let mut res = post();

        debug!("proxy_get_online_proxy - response code: {}", res.status().as_u16());

        if res.status().as_u16() == 200 {
            let recv: GetOnlinePorxyRecv = serde_json::from_str(
                &res.text().map_err(Error::Reqwest)?
            ).map_err(Error::GetOnlineProxyParseJsonStr)?;

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
                            return Err(Error::GetOnlineProxyInvalidData);
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
                return Ok(());
            }
            else {
                if let Some(msg) = recv.msg {
                    return Err(Error::GetOnlineProxy(msg));
                }
            }
        }
        else {
            let mut err_msg = "Unknown reason.".to_string();
            if let Ok(msg) = res.text() {
                err_msg = msg;
            }
            return Err(Error::GetOnlineProxy(
                format!("Code:{} Msg:{}", res.status().as_u16(), err_msg).to_string()));
        }
        return Err(Error::GetOnlineProxy("Unknown reason.".to_string()));
    }

    pub fn proxy_heart_beat(&self, info: &Info) -> Result<()> {
        let post = "/vppn/api/v2/proxy/hearBeat";
        let url = self.url.to_string() + post;
        let data = Heartbeat::new_from_info(info).to_json();
        let cookie = info.proxy_info.cookie.clone();

        debug!("proxy_heart_beat - request url: {}",url);
        debug!("proxy_heart_beat - request data: {}",data);

        let post = || {
            let start = Instant::now();
            loop {
                match url_post(&url, &data, &cookie) {
                    Ok(x) => return Some(x),
                    Err(e) => {
                        error!("proxy_heart_beat - response {:?}", e);

                        if Instant::now() - start > Duration::from_secs(10) {
                            return None;
                        };
                        continue;
                    }
                };
            }
        };

        let mut res = match post() {
            Some(x) => x,
            None => return Err(Error::HeartbeatTimeout),
        };

        debug!("proxy_heart_beat - response code: {}",res.status().as_u16());

        if res.status().as_u16() == 200 {
            let data = res.text().map_err(Error::Reqwest)?;
            debug!("proxy_heart_beat - response data: {:?}", data);

            let recv: Recv = serde_json::from_str(&data)
                .map_err(Error::HeartbeatJsonStr)?;

            if recv.code == 200 {
                return Ok(());
            }
        }
        return Err(Error::HeartbeatFailed);
    }
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct JavaRegister {
    authId:                 Option<String>,
    authType:               Option<String>,
    area:                   Option<String>,
    countryCode:            Option<String>,
    proxyIp:                Option<String>,
    pubKey:                 Option<String>,
    os:                     Option<String>,
    serverType:             Option<String>,
    sshPort:                Option<String>,
    latitude:               Option<String>,
    longitude:              Option<String>,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
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
        return serde_json::to_string(self).unwrap();
    }
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct VecProxy {
    inner: Vec<Proxy>,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
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
        return serde_json::to_string(self).unwrap();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
        return serde_json::to_string(self).unwrap();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Login {
    code:    i32,
    data:    LoginUser,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginUser {
    pub userid:                         String,
    pub username:                       String,
    pub useremail:                      String,
//    pub photo:                          Option<String>,
//    pub devices:                        Option<Vec<Device>>,
    pub enable_autogroup:               bool,
    pub enable_autoothergroup:          bool,
    pub enable_autonetworking:          bool,
    pub invitetime:                     String,
}

//#[derive(Clone, Debug, Serialize, Deserialize)]
//pub struct Device {
//    deviceid:    Option<String>,
//    devicename:  Option<String>,
//    devicetype:  Option<i32>,
//    lan:         Option<String>,
//    wan:         Option<String>,
//    ip:          Option<String>,
//}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Recv {
    code:        i32,
    msg:         Option<String>,
    data:        Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RegisterRecv {
    code:        i32,
    msg:         Option<String>,
    data:        Option<JavaRegister>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetOnlinePorxyRecv {
    code:        i32,
    msg:         Option<String>,
    data:        Vec<Proxy>,
}