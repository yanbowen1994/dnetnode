use crate::info::get_info;
use crate::settings::get_settings;
use crate::rpc::client::rpc_client::types::teams::{JavaResponseTeamMember, JavaResponseDeviceProxy};
use super::post;
use super::{Error, Result};

pub(super) fn out_team(team_id: &str) -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vppn/api/v2/client/outteam";

    let device_id;
    let cookie;
    {
        let info = get_info().lock().unwrap();
        device_id = info.client_info.uid.clone();
        cookie = info.client_info.cookie.clone();
    }
    let data = RequestJoinTeam {
        teamid:   team_id.to_owned(),
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
                            error!("key_report response code: {} msg: {:?}", recv.code, recv.msg);
                            return Err(Error::http(recv.code));
                        }
                    }
                    else if let Ok(recv) = serde_json::from_str(res_data) {
                        let recv: JavaResponseAlreadyIn = recv;
                        if recv.code == 931 {
                            return Ok(());
                        }
                    }
                    else {
                        error!("out_team - response can't parse: {:?}", res_data);
                    }
                }
                else {
                    error!("{:?}", res);
                }
            }
            else {
                error!("{:?}", res);
            }

            return Err(Error::out_team);
        })
}

#[derive(Debug, Serialize, Deserialize)]
struct RequestJoinTeam {
    teamid:   String,
    deviceid: String,
}

impl RequestJoinTeam {
    fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JavaResponse {
    code: i32,
    data: Option<JavaResponseTeamMember>,
    msg:  Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JavaResponseAlreadyIn {
    code: i32,
    data: Option<Vec<JavaResponseDeviceProxy>>,
    msg:  Option<String>,
}