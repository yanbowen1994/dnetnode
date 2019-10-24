//! upload proxy status
use std::net::IpAddr;
use std::str::FromStr;
use std::time::{Instant, Duration};

use tinc_plugin::{TincOperatorError, ConnectTo};

use crate::settings::{Settings, get_settings};
use crate::info::{get_info, get_mut_info};
use crate::tinc_manager;
use crate::rpc::http_post::url_post;
use super::post::post;

const HEART_BEAT_TIMEOUT: u64 = 10;

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

    #[error(display = "Heartbeat can not parse json str.")]
    HeartbeatJsonStr(#[error(cause)] serde_json::Error),

    #[error(display = "Heartbeat timeout.")]
    HeartbeatTimeout,

    #[error(display = "Heartbeat failed.")]
    HeartbeatFailed,

    #[error(display = "reqwest::Error.")]
    Reqwest(#[error(cause)] reqwest::Error),

    #[error(display = "operator::Error.")]
    TincOperator(#[error(cause)] TincOperatorError),
}

#[derive(Debug)]
pub(super) struct RpcClient {
    username:   String,
    password:   String,
}
impl RpcClient {
    pub fn new() -> Self {
        let settings = get_settings();
        let username = settings.common.username.to_owned();
        let password = settings.common.password.to_owned();
        RpcClient {
            username,
            password,
        }
    }

    pub fn proxy_login(&self,
                       settings:    &Settings,
    ) -> Result<()> {
        let url = get_settings().common.conductor_url.clone() + "/login";
        let data = User::new_from_settings(settings).to_json();

        debug!("proxy_login - request url: {} ",url);
        debug!("proxy_login - request data:{}",data);

        let mut res = post(&url, &data, "")?;

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
                get_mut_info().lock().unwrap().proxy_info.cookie = cookie_str.to_string();
            }

            let res_data = res.text().map_err(Error::Reqwest)?;
            debug!("proxy_login - response data: {:?}", res_data);
            let _login: Login = serde_json::from_str(&res_data)
                .map_err(|e|{
                    error!("proxy_login - response data: {:?}", res_data);
                    Error::LoginParseJsonStr(e)
                })?;

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
    }

    pub fn proxy_register(&self) -> Result<()> {
        let url = get_settings().common.conductor_url.clone() + "/vppn/api/v2/proxy/register";
        let data = Register::new_from_info().to_json();
        let cookie;
        {
            cookie = get_info().lock().unwrap().proxy_info.cookie.clone();
        }
        debug!("proxy_register - request url: {}", url);
        debug!("proxy_register - request data: {}", data);
	    let mut res = post(&url, &data, &cookie)?;

        debug!("proxy_register - response code: {}", res.status().as_u16());

        if res.status().as_u16() == 200 {
            let res_data = &res.text().map_err(Error::Reqwest)?;
            let recv: RegisterRecv = serde_json::from_str(res_data)
                .map_err(|e|{
                error!("proxy_register - response data: {}", res_data);
                Error::RegisterParseJsonStr(e)
            })?;

            debug!("proxy_register - response data: {:?}", recv);

            if recv.code == 200 {
                get_mut_info().lock().unwrap().proxy_info.isregister = true;
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

    pub fn proxy_get_online_proxy(&self) -> Result<bool> {
        // config change need restart tinc
        let mut config_change = false;

        let settings = get_settings();
        let url = settings.common.conductor_url.clone()
            + "/vppn/api/v2/proxy/getonlineproxy";

        let data = Register::new_from_info().to_json();
        let cookie;
        {
            cookie = get_info().lock().unwrap().proxy_info.cookie.clone();
        }
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
            let res_data = &res.text().map_err(Error::Reqwest)?;
            let recv: GetOnlinePorxyRecv = serde_json::from_str(res_data)
                .map_err(|e|{
                    error!("proxy_get_online_proxy - response data: {}", res_data);
                    Error::GetOnlineProxyParseJsonStr(e)
            })?;

            if recv.code == 200 {
                let mut info = get_mut_info().lock().unwrap();
                let _ = info.tinc_info.load_local();
                let local_pub_key = info.tinc_info.pub_key.clone();
                let mut local_vip = info.tinc_info.vip.clone();
                let mut local_ip = info.proxy_info.ip.clone();
                std::mem::drop(info);

                let proxy_vec: Vec<Proxy> = recv.data;

                let mut other_proxy = vec![];

                let tinc = tinc_manager::TincOperator::new();

                for proxy in proxy_vec {
                    if proxy.pubkey.to_string() == local_pub_key {
                        if let Ok(vip) = IpAddr::from_str(&proxy.vip) {
                            if local_vip != Some(vip) {
                                config_change = true;
                                local_vip = Some(vip);
                            }
                        }
                        else {
                            return Err(Error::GetOnlineProxy("Response not include proxy_vip or proxy_vip can't be parse.".to_owned()));
                        }

                        if let Ok(ip) = IpAddr::from_str(&proxy.ip) {
                            if local_ip != Some(ip) {
                                config_change = true;
                                local_ip = Some(ip);
                            }
                        }
                        else {
                            return Err(Error::GetOnlineProxy("Response not include proxy_ip or proxy_ip can't be parse.".to_owned()));
                        }
                    }
                    else {
                        if let Ok(other_ip) = IpAddr::from_str(&proxy.ip) {
                            if let Ok(other_vip) = IpAddr::from_str(&proxy.vip) {
                                let other = ConnectTo::from(other_ip, other_vip, proxy.pubkey);
                                tinc.set_hosts(
                                    true,
                                    &other.ip.to_string(),
                                    &("Address=".to_string() +
                                        &(&other.ip.clone()).to_string() +
                                        "\n" +
                                        &other.pubkey +
                                        "Port=50069")
                                )
                                    .map_err(Error::TincOperator)?;
                                other_proxy.push(other);
                                continue
                            }
                        }
                        error!("proxy_get_online_proxy - One proxy data invalid");
                    }
                }

                let mut info = get_mut_info().lock().unwrap();
                info.tinc_info.connect_to = other_proxy;
                info.proxy_info.ip = local_ip;
                info.tinc_info.vip = local_vip;
                std::mem::drop(info);

                return Ok(config_change);
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

    pub fn proxy_heart_beat(&self) -> Result<()> {
        let url = get_settings().common.conductor_url.clone()
            + "/vppn/api/v2/proxy/hearBeat";
        let data = Heartbeat::new_from_info().to_json();
        let cookie;
        {
            cookie = get_info().lock().unwrap().proxy_info.cookie.clone();
        }

        debug!("proxy_heart_beat - request url: {}",url);
        debug!("proxy_heart_beat - request data: {}",data);

        let post = || {
            let start = Instant::now();
            loop {
                match url_post(&url, &data, &cookie) {
                    Ok(x) => return Some(x),
                    Err(e) => {
                        error!("proxy_heart_beat - response {:?}", e);

                        if Instant::now() - start > Duration::from_secs(HEART_BEAT_TIMEOUT) {
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
            let res_data = &res.text().map_err(Error::Reqwest)?;;
            debug!("proxy_heart_beat - response data: {:?}", res_data);

            let recv: Recv = serde_json::from_str(&res_data)
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
    proxyIp: String,
    pubKey: String,
    os: String,
    server_type: String,
    ssh_port: String,
}
impl Register {
    fn new_from_info() -> Self {
        let info = get_info().lock().unwrap();
        Register {
            auth_id: info.proxy_info.uid.to_string(),
            auth_type: info.proxy_info.auth_type.to_string(),
            proxyIp: "".to_owned(),
            pubKey: info.tinc_info.pub_key.to_string(),
            os: info.proxy_info.os.to_string(),
            server_type: info.proxy_info.server_type.to_string(),
            ssh_port: info.proxy_info.ssh_port.to_string(),
        }
    }

    fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetOnlinePorxyRecv {
    code:        i32,
    msg:         Option<String>,
    data:        Vec<Proxy>,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Proxy {
    id:                 String,
    ip:                 String,
    country:            Option<String>,
    region:             Option<String>,
    city:               Option<String>,
    username:           Option<String>,
    teamcount:          Option<u32>,
    ispublic:           Option<bool>,
    vip:                String,
    pubkey:             String,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Heartbeat {
    authID: String,
    connections:    u32,
    edges:          u32,
    nodes:          u32,
    pubKey:         String,
}
impl Heartbeat {
    fn new_from_info() -> Self {
        let mut info = get_mut_info().lock().unwrap();
        let _ = info.tinc_info.flush_connections();
        Heartbeat {
            authID:         info.proxy_info.uid.to_string(),
            connections:    info.tinc_info.connections,
            edges:          info.tinc_info.edges,
            nodes:          info.tinc_info.nodes,
            pubKey:         info.tinc_info.pub_key.to_string(),
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
            username: settings.common.username.clone(),
            password: settings.common.password.clone(),
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
