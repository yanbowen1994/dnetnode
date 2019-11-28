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

    let device_name = info.client_info.device_name.clone();
    let mut proxy_ids = vec![];
    for connect_to in &info.tinc_info.connect_to {
        proxy_ids.push(connect_to.id.to_owned());
    }
    let pubkey = info.tinc_info.pub_key.clone();

    std::mem::drop(info);

    let data = json!({
        "deviceSerial":         device_name,
        "proxyIds":              proxy_ids,
        "pubKey":               pubkey,
    }).to_string();

    info!("request {}", data);

    let _ = post(&url, &data)?;

    Ok(())
}