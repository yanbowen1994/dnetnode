use std::sync::{Arc, Mutex};
use crate::info::{Info, get_info};
use super::types::DeviceId;
use super::post;
use super::{Error, Result};
use crate::settings::get_settings;

pub(super) fn device_select_proxy() -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vppn/api/v2/client/deviceselectproxy";

    let data;
    let cookie;
    {
        let info = get_info().lock().unwrap();
        if info.tinc_info.connect_to.len() > 0 {
            let proxy_ip = info.tinc_info.connect_to[0].ip.to_string();
            let device_id = info.client_info.uid.clone();
            let pubkey = info.tinc_info.pub_key.clone();
            cookie = info.client_info.cookie.clone();
            data = DeviceSelectProxy {
                proxyip:    proxy_ip,
                deviceid:   device_id,
                pubkey,
            }.to_json();
        }
        else {
            return Err(Error::NoUsableProxy);
        }
    }

    post(&url, &data, &cookie)
        .and_then(|mut res| {
            if res.status().as_u16() == 200 {
                if let Ok(res_data) = &res.text() {
                    if let Ok(recv) = serde_json::from_str(res_data) {
                        let recv: JavaResponse = recv;
                        if recv.code == 200 {
                            return Ok(());
                        }
                        else {
                            error!("device_select_proxy response msg: {:?}", recv.msg);
                        }
                    }
                    else {
                        error!("device_select_proxy - response can't parse: {:?}", res_data);
                    }
                }
                else {
                    error!("device_select_proxy - {:?}", res);
                }
            }
            else {
                error!("device_select_proxy - {:?}", res);
            }

            return Err(Error::device_select_proxy);
        })
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct DeviceSelectProxy {
    deviceid:   String,
    proxyip:    String,
    pubkey:     String,
}

impl DeviceSelectProxy {
    fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JavaResponse {
    code: i32,
    data: Option<String>,
    msg:  Option<String>,
}