use std::net::IpAddr;
use std::str::FromStr;

use serde_json;

use crate::settings::get_settings;

use crate::rpc::http_request::post;
use crate::rpc::{Error, Result};
use crate::info::get_mut_info;
use crate::rpc::client::rpc_client::types::JavaDevice;

pub fn device_add() -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vlan/device/add";

    let data = JavaDevice::new().to_json();

    info!("Request {}", data);

    let res_data = post(&url, &data.to_string())?;
    info!("Response {:?}", res_data.to_string());
    let res_device: JavaDevice = serde_json::from_value(res_data.clone())
        .map_err(|e|{
            Error::ResponseParse(format!("err: {:?}\n{:?}", e, res_data.to_string()))
        })?;


    let vip = res_device.ip
        .and_then(|vip|IpAddr::from_str(&vip).ok())
        .ok_or(Error::ResponseParse("device_add response vip.".to_owned()))?;

    let mut info = get_mut_info().lock().unwrap();
    info.tinc_info.vip = Some(vip);
    Ok(())
}