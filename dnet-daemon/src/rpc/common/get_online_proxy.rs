use std::net::IpAddr;
use std::str::FromStr;

use serde_json;
use tinc_plugin::ConnectTo;

use crate::settings::get_settings;
use crate::info::get_info;
use crate::rpc::{Error, Result};
use crate::rpc::http_request::get_mutipage;

pub fn get_online_proxy() -> Result<Vec<ConnectTo>> {
    let url = get_settings().common.conductor_url.clone()
        + "/vlan/proxy/queryAllOnline";
    let res = get_mutipage(&url)?;
    info!("Response: {:?}", res);
    let mut proxy_vec: Vec<GetProxyResponse> = vec![];
    for record in res {
        let proxy = serde_json::from_value(record)
            .map_err(|e|Error::ResponseParse(format!("GetProxyResponse error: {:?}", e)))?;
        proxy_vec.push(proxy);
    }

    let connect_to = parse_response(proxy_vec)?;
    Ok(connect_to)
}

fn parse_response(proxy_vec: Vec<GetProxyResponse>) -> Result<Vec<ConnectTo>> {
    let local_vip = get_info().lock().unwrap().tinc_info.vip.clone();

    let mut connect_to: Vec<ConnectTo> = vec![];
    for proxy in proxy_vec {
        let (proxy_id, proxy_ip, proxy_vip, proxy_port, proxy_pubkey)
            = match get_remote_info(&proxy) {
            Some(x) => x,
            None => continue,
        };

        if local_vip != Some(proxy_vip) {
            let other = ConnectTo::from(proxy_id, proxy_ip, proxy_vip, proxy_port, proxy_pubkey);
            connect_to.push(other);
        }
    }

    Ok(connect_to)
}

fn get_remote_info(proxy: &GetProxyResponse) -> Option<(String, IpAddr, IpAddr, u16, String)> {
    let proxy_id = match proxy.id.clone() {
        Some(x) => x,
        None => return None,
    };

    let proxy_ip = match proxy.ip
        .clone()
        .and_then(|ip| {
            IpAddr::from_str(&ip).ok()
        }) {
        Some(x) => x,
        None => return None,
    };

    let proxy_vip = match proxy.vip
        .clone()
        .and_then(|vip| {
            IpAddr::from_str(&vip).ok()
        }) {
        Some(x) => x,
        None => return None,
    };

    let proxy_port = match &proxy.tincPort {
        Some(x) => x.clone(),
        None => return None,
    };

    let proxy_pubkey = match &proxy.pubkey {
        Some(x) => x.clone(),
        None => return None,
    };

    Some((proxy_id, proxy_ip, proxy_vip, proxy_port, proxy_pubkey))
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetProxyResponse {
    authId:                Option<String>,
    city:                  Option<String>,
    companyId:             Option<String>,
    country:               Option<String>,
    createBy:              Option<String>,
    createTime:            Option<String>,
    id:                    Option<String>,
    ip:                    Option<String>,
    latitude:              Option<String>,
    longitude:             Option<String>,
    pubkey:                Option<String>,
    publicFlag:            Option<bool>,
    serverPort:            Option<u16>,
    status:                Option<i32>,
    tincPort:              Option<u16>,
    updateBy:              Option<String>,
    updateTime:            Option<String>,
    userId:                Option<String>,
    vip:                   Option<String>,
}