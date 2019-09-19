use crate::info::get_mut_info;
use super::{Error, Result};
use crate::settings::get_settings;
use std::time::{Instant, Duration};
use crate::net_tool::url_post;
use crate::rpc::client::rpc_client::types::Recv;

const HEART_BEAT_TIMEOUT: u64 = 10;

pub fn client_heartbeat() -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vppn/api/v2/client/hearBeat";
    let data = ClientHeartbeat::new_from_info().to_json();

    debug!("client_heart_beat - request url: {}",url);
    debug!("client_heart_beat - request data: {}",data);

    let post = || {
        let start = Instant::now();
        loop {
            match url_post(&url, &data, "0cde13b523sf9aa5a403dc9f5661344b91d77609f70952eb488f31641") {
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
    fn new_from_info() -> Self {
        let mut info = get_mut_info().lock().unwrap();
        let _ = info.tinc_info.flush_connections();
        Self {
            deviceid:       info.client_info.uid.clone(),
            devicetype:     "1".to_owned(),
            lan:            "".to_owned(),
            lan_info:       "".to_owned(),
            proxyip:        "".to_owned(),
            status:         "1".to_owned(),
            teamid:         "1".to_owned(),
            wan:            "".to_owned(),
        }
    }

    fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}