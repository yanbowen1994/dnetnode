mod center_get_team_info;
mod heartbeat;
use heartbeat::proxy_heartbeat;
mod proxy_add;
use proxy_add::proxy_add;

use tinc_plugin::ConnectTo;
use crate::rpc::common::login::login;
use crate::rpc::common::get_online_proxy;
use crate::rpc::Result;
use crate::info::get_mut_info;
use crate::tinc_manager::TincOperator;

#[derive(Debug)]
pub struct RpcClient;

impl RpcClient {
    pub fn new() -> Self {
        Self
    }

    pub fn center_get_team_info(&self) -> Result<()> {
        center_get_team_info::center_get_team_info()
    }

    pub fn proxy_heartbeat(&self) -> Result<()> {
        proxy_heartbeat()
    }

    pub fn proxy_login(&self) -> Result<()> {
        login()
    }

    pub fn proxy_add(&self) -> Result<()> {
        proxy_add()
    }

    pub fn proxy_get_online_proxy(&self) -> Result<Vec<ConnectTo>> {
        get_online_proxy::get_online_proxy()
    }

    pub fn init_connect_to(&self, connect_to: Vec<ConnectTo>) {
        let mut info = get_mut_info().lock().unwrap();
        info.tinc_info.connect_to = connect_to;
    }

    pub fn add_connect_to_host(&self, connect_to: Vec<ConnectTo>) {
        let tinc = TincOperator::new();
        for host in connect_to.clone() {
            let _ = tinc.set_hosts(
                    Some((host.ip, host.port)),
                    host.vip,
                    &host.pubkey,
                )
                .map_err(|e| {
                    error!("add_connect_to_host failed {:?} error:{:?}", host, e);
                });
        }
        let mut info = get_mut_info().lock().unwrap();
        info.tinc_info.connect_to = connect_to;
    }
}