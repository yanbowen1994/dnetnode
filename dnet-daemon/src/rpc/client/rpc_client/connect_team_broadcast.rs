use crate::info::get_info;
use super::types::DeviceId;
use super::post;
use super::{Error, Result};
use crate::settings::get_settings;

pub fn connect_team_broadcast() -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vppn/api/v2/client/heartbeat";
    let info = get_info().lock().unwrap();
    let deviceid = info.client_info.uid.clone();
    let deviceip = info.tinc_info.vip.clone().to_string();
    let cookie = info.client_info.cookie.clone();

    let running_teams: Vec<String> = info.client_info.running_teams
        .iter()
        .map(|team|team.team_id.clone())
        .collect();
    std::mem::drop(info);

    debug!("client_heart_beat - request url: {}", url);
    for teamid in running_teams {
        let data = Broadcast {
            deviceid:     deviceid.clone(),
            deviceip:     deviceip.clone(),
            status:       "1".to_owned(),
            teamid:       teamid.clone(),
        }.to_json();

        debug!("client_heart_beat - request data: {}", data);
        connect_team_broadcast_inner(&url, &data, &cookie)?;
    }
    Ok(())
}

fn connect_team_broadcast_inner(url: &str, data: &str, cookie: &str) -> Result<()> {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Broadcast {
    deviceid:     String,
    deviceip:     String,
    status:       String,
    teamid:       String,
}

impl Broadcast {
    fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}