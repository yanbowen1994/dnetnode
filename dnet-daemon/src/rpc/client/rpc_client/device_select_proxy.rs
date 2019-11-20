use serde_json::json;

use crate::info::get_info;
use crate::settings::get_settings;
use crate::rpc::http_request::post;
use crate::rpc::{Error, Result};

pub(super) fn device_select_proxy() -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vlan/device/proxy/selectDeviceByProxy";

    let info = get_info().lock().unwrap();

    if info.tinc_info.connect_to.len() == 0 {
        return Err(Error::http(511));
    }

    let device_serial = info.client_info.uid.clone();
    let proxy_ip = info.tinc_info.connect_to[0].ip.clone().to_string();
    let proxy_port = info.tinc_info.connect_to[0].port;
    let pubkey = info.tinc_info.connect_to[0].pubkey.clone();

    std::mem::drop(info);

    let data = json!({
        "deviceSerial":         device_serial,
        "proxyIp":              proxy_ip,
        "proxyPort":            proxy_port,
        "pubKey":               pubkey,
    }).to_string();

    let _ = post(&url, &data)?;

    Ok(())
}