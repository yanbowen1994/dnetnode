mod device_add;
mod device_select_proxy;
mod get_users_by_team;
#[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
mod connect_disconnect_team;
mod fresh_route;
#[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
mod join_team;
#[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
mod out_team;
#[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
mod search_team_by_user;
mod select_proxy;
mod types;
mod search_team_by_mac;

#[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
extern crate router_plugin;

use tinc_plugin::ConnectTo;
use crate::info::UserInfo;
use crate::rpc::common::login::login;
use crate::rpc::common::get_online_proxy;

use crate::rpc::{Result, Error};
pub use select_proxy::select_proxy;

#[derive(Debug)]
pub struct RpcClient;

impl RpcClient {
    pub fn new() -> Self {
        RpcClient {}
    }

    pub fn client_login(&self) -> Result<()> {
        login()
    }

    pub fn device_add(&self) -> Result<()> {
        device_add::device_add()
    }

    pub fn client_get_online_proxy(&self) -> Result<Vec<ConnectTo>> {
        get_online_proxy::get_online_proxy()
    }

    pub fn device_select_proxy(&self) -> Result<()> {
        device_select_proxy::device_select_proxy()
    }

    pub fn search_team_by_mac(&self) -> Result<()> {
        search_team_by_mac::search_team_by_mac()
    }

    pub fn get_users_by_team(&self, team_id: &str) -> Result<Vec<UserInfo>> {
        get_users_by_team::get_users_by_team(team_id)
    }
}

#[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
impl RpcClient {
    pub fn connect_team(&self, team_id: &str) -> Result<()> {
        connect_disconnect_team::connect_disconnect_team(team_id, true)
    }

    pub fn disconnect_team(&self, team_id: &str) -> Result<()> {
        connect_disconnect_team::connect_disconnect_team(team_id, false)
    }

    pub fn join_team(&self, team_id: &str) -> Result<()> {
        if let Err(e) = join_team::join_team(team_id) {
            match e {
                Error::http(code) => {
                    if code != 645 {
                        return Err(Error::http(code));
                    }
                }
                _ => return Err(e),
            }
        }
        self.connect_team(team_id)
    }

    pub fn out_team(&self, team_id: &str) -> Result<()> {
        out_team::out_team(team_id)
    }

    pub fn search_team_by_user(&self) -> Result<()> {
        search_team_by_user::search_team_by_user()
    }
}