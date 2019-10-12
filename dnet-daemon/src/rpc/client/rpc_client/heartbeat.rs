use crate::info::get_info;
use super::{Error, Result};
use crate::settings::get_settings;
use std::time::{Instant, Duration};
use crate::rpc::http_post::url_post;
use crate::rpc::client::rpc_client::types::Recv;

const HEART_BEAT_TIMEOUT: u64 = 10;

pub fn client_heartbeat() -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vppn/api/v2/client/heartbeat";
    let info = get_info().lock().unwrap();
    let deviceid = info.client_info.uid.clone();
    let cookie = info.client_info.cookie.clone();

    let running_teams: Vec<String> = info.client_info.running_teams
        .iter()
        .map(|team|team.team_id.clone())
        .collect();
    std::mem::drop(info);
    for teamid in running_teams {
        let data = ClientHeartbeat {
            deviceid:       deviceid.clone(),
            devicetype:     "1".to_owned(),
            lan:            "".to_owned(),
            lan_info:       "".to_owned(),
            proxyip:        "".to_owned(),
            status:         "1".to_owned(),
            teamid,
            wan:            "".to_owned(),
        }.to_json();

        debug!("client_heart_beat - request url: {}", url);
        debug!("client_heart_beat - request data: {}", data);

        heartbeat_inner(&url, &data, &cookie)?;
    }
    Ok(())
}

fn heartbeat_inner(url: &str, data: &str, cookie: &str) -> Result<()> {
    let post = || {
        let start = Instant::now();
        loop {
            match url_post(&url, &data, &cookie) {
                Ok(x) => return Some(x),
                Err(e) => {
                    error!("client_heart_beat - response {:?}", e);

                    if Instant::now() - start > Duration::from_secs(HEART_BEAT_TIMEOUT) {
                        return None;
                    };
                    continue;
                }
            };
        }
    };

    let mut res = match post() {
        Some(x) => x,
        None => return Err(Error::HeartbeatTimeout),
    };

    debug!("client_heart_beat - response code: {}",res.status().as_u16());

    if res.status().as_u16() == 200 {
        let res_data = &res.text().map_err(Error::Reqwest)?;;
        debug!("client_heart_beat - response data: {:?}", res_data);

        let recv: Recv = serde_json::from_str(&res_data)
            .map_err(Error::HeartbeatJsonStr)?;

        if recv.code == 200 {
            return Ok(());
        }
        else {
            error!("client_heart_beat - code:{:?} error:{:?}", recv.code, recv.msg);
        }
    }
    return Err(Error::HeartbeatFailed);
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ClientHeartbeat {
    deviceid:            String,
    devicetype:          String,
    lan:                 String,
    lan_info:            String,
    proxyip:             String,
    status:              String,
    teamid:              String,
    wan:                 String,
}

impl ClientHeartbeat {
    fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}