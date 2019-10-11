//! upload client status
use std::thread::sleep;

use reqwest::Response;
#[cfg(target_arch = "arm")]
extern crate router_plugin;

use crate::rpc::http_post::url_post;

use super::{Error, Result};
use super::login::login;
use super::search_team_by_mac::search_team_by_mac;
use super::binding_device::binding_device;
use super::heartbeat::client_heartbeat;
use super::key_report::client_key_report;
use crate::rpc::client::rpc_client::search_user_team::search_user_team;
use crate::rpc::client::rpc_client::join_team::join_team;
use super::get_online_proxy::client_get_online_proxy;
use super::device_select_proxy::device_select_proxy;
use tinc_plugin::ConnectTo;

pub(super) fn post(url: &str, data: &str, uid: &str) -> Result<Response> {
    let mut wait_sec = 0;
    //ApiKey
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

    pub fn client_key_report(&self) -> Result<()> {
        client_key_report()
    }

    pub fn client_get_online_proxy(&self) -> Result<Vec<ConnectTo>> {
        client_get_online_proxy()
    }

    pub fn device_select_proxy(&self) -> Result<()> {
        device_select_proxy()
    }

    pub fn client_heartbeat(&self) -> Result<()> {
        client_heartbeat()
    }

    pub fn search_team_by_mac(&self) -> Result<bool> {
        search_team_by_mac()
    }

    pub fn search_user_team(&self) -> Result<bool> {
        search_user_team()
    }

    pub fn binding_device(&self) -> Result<()> {
        binding_device()
    }
}

#[cfg(target_arc = "test")]
impl RpcClient {
    pub fn join_team(&self, team_id: String) -> Result<()> {
        join_team(team_id)
    }

    pub fn search_user_team(&self) -> Result<()> {
        search_user_team()
    }
}