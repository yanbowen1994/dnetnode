//! upload client status
use std::net::IpAddr;
use std::str::FromStr;
use std::time::{Instant, Duration};
use std::thread::sleep;

use reqwest::Response;
use tinc_plugin::TincOperatorError;
#[cfg(target_arch = "arm")]
extern crate router_plugin;

use crate::net_tool::url_post;
use crate::settings::{Settings, get_settings};
use crate::info::{Info, get_info, get_mut_info};
use crate::tinc_manager;
use mac_address::get_mac_address;
use std::sync::{Mutex, Arc};

use super::{Error, Result};
use super::login;
use super::search_team_by_mac;
use super::binding_device;
use super::client_heartbeat;

pub(super) fn post(url: &str, data: &str, uid: &str) -> Result<Response> {
    let mut wait_sec = 0;
    loop {
        let _res = match url_post(&url, &data, uid) {
            Ok(x) => return Ok(x),
            Err(e) => {
                error!("post - response {:?}", e);
                sleep(std::time::Duration::from_secs(wait_sec));
                if wait_sec < 5 {
                    wait_sec += 1;
                }
                else {
                    return Err(Error::Reqwest(e))
                }
                continue;
            }
        };
    }
}

#[derive(Debug)]
pub struct RpcClient;

impl RpcClient {
    pub fn new() -> Self {
        RpcClient {}
    }

    pub fn client_login(&self) -> Result<()> {
        login()
    }

    pub fn binding_device(&self) -> Result<()> {
        binding_device()
    }

    pub fn search_team_by_mac(&self) -> Result<()> {
        search_team_by_mac()
    }

    pub fn client_heartbeat(&self) -> Result<()> {
        client_heartbeat()
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
struct DeviceId {
    deviceid: String,
}

impl DeviceId {
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

//#[derive(Clone, Debug, Serialize, Deserialize)]
//pub struct Device {
//    deviceid:    Option<String>,
//    devicename:  Option<String>,
//    devicetype:  Option<i32>,
//    lan:         Option<String>,
//    wan:         Option<String>,
//    ip:          Option<String>,
//}

