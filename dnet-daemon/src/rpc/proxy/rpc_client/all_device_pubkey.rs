use std::net::IpAddr;
use std::str::FromStr;

use tinc_plugin::{TincTeam, PID_FILENAME};
use crate::settings::get_settings;
use crate::tinc_manager::TincOperator;
use crate::rpc::{Error, Result};
use crate::rpc::http_request::get;

pub fn all_device_pubkey() -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vlan/device/getAllDevicePubkeys";
    let res_data = get(&url)?;

    let tinc = TincOperator::new();
    for (vip, pubkey_value) in res_data.as_object()
    .ok_or(Error::ResponseParse(res_data.to_string()))? {
        if let Ok(vip) = IpAddr::from_str(vip) {
            if let Some(pubkey) = pubkey_value.as_str() {
                tinc.set_hosts(None, vip, pubkey);
            }
        }
    }

    return Ok(())
}