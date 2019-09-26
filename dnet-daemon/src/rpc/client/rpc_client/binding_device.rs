use crate::info::get_info;
use super::types::DeviceId;
use super::post;
use super::{Error, Result};
use crate::settings::get_settings;

pub(super) fn binding_device() -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vppn/api/v2/client/bindingdevice";

    let device_id;
    let cookie;
    {
        let info = get_info().lock().unwrap();
        device_id = info.client_info.uid.clone();
        cookie = info.client_info.cookie.clone();
    }
    let data = DeviceId {
        deviceid: device_id,
    }.to_json();

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
                            if recv.msg == Some("The device has been bound by other users.".to_owned()) {
                                return Ok(());
                            }
                            error!("binding_device response msg: {:?}", recv.msg);
                        }
                    }
                    else {
                        error!("binding_device - response can't parse: {:?}", res_data);
                    }
                }
                else {
                    error!("{:?}", res);
                }
            }
            else {
                error!("{:?}", res);
            }

            return Err(Error::search_team_by_mac);
        })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JavaResponse {
    code: i32,
    data: Option<String>,
    msg:  Option<String>,
}