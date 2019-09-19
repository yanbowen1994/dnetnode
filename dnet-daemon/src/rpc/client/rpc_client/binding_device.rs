use std::sync::{Arc, Mutex};
use crate::info::{Info, get_info};
use super::types::DeviceId;
use super::post;
use super::{Error, Result};
use crate::settings::get_settings;

pub(super) fn binding_device() -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vppn/api/v2/client/bindingdevice";

    let deviceid;
    {
        deviceid = get_info().lock().unwrap().client_info.uid.clone();
    }
    let data = DeviceId {
        deviceid,
    }.to_json();

    post(&url, &data, "")
        .and_then(|mut res| {
            if res.status().as_u16() == 200 {
                if let Ok(res_data) = &res.text() {
                    if let Ok(recv) = serde_json::from_str(res_data) {
                        let recv: JavaResponse = recv;
                        if recv.code == 200 {
                            return Ok(());
                        }
                    }
                    else {
                        debug!("binding_device - response can't parse: {:?}", res_data);
                    }
                }
            }
            error!("{:?}", res);
            return Err(Error::search_team_by_mac);
        })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JavaResponse {
    code: i32,
    data: Option<String>,
    msg:  Option<String>,
}