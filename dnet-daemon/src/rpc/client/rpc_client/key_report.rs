use crate::info::get_info;
use crate::settings::get_settings;
use super::post;
use super::{Error, Result};

pub(super) fn client_key_report() -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vppn/api/v2/client/keyreport";

    let cookie;
    {
        cookie = get_info().lock().unwrap().client_info.cookie.clone();
    }

    let data = KeyReport::new_from_info().to_json();

    post(&url, &data, &cookie)
        .and_then(|mut res| {
            if res.status().as_u16() == 200 {
                if let Ok(res_data) = &res.text() {
                    if let Ok(recv) = serde_json::from_str(res_data) {
                        let recv: JavaResponse = recv;
                        info!("client_key_report response: {:?}", recv);
                        if recv.code == 200 {
                            return Ok(());
                        }
                        else {
                            error!("key_report response msg: {:?}", recv.msg);
                            return Err(Error::http(recv.code));
                        }
                    }
                    else {
                        error!("key_report - response can't parse: {:?}", res_data);
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
struct KeyReport {
    deviceid:           String,
    pubkey:             String,
    devicetype:         String,
    lan:                String,
    wan:                String,
    devicename:         String,
}

impl KeyReport {
    fn new_from_info() -> Self {
        let info = get_info().lock().unwrap();
        KeyReport {
            deviceid: info.client_info.uid.clone(),
            pubkey: info.tinc_info.pub_key.clone(),
            devicetype: info.client_info.devicetype.clone(),
            lan: info.client_info.lan.clone(),
            wan: info.client_info.wan.clone(),
            devicename: info.client_info.devicename.clone(),
        }
    }

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