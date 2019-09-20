use std::sync::{Arc, Mutex};
use crate::info::{Info, get_info};
use super::types::DeviceId;
use super::post;
use super::{Error, Result};
use crate::settings::get_settings;
use crate::rpc::client::rpc_client::types::teams::{JavaResponseTeam, JavaResponseTeamMember, JavaResponseDeviceProxy};
use dnet_types::team::DeviceProxy;

pub(super) fn join_team(team_id: String) -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vppn/api/v2/client/jointeam";

    let device_id;
    let cookie;
    {
        let info = get_info().lock().unwrap();
        device_id = info.client_info.uid.clone();
        cookie = info.client_info.cookie.clone();
    }
    let data = RequestJoinTeam {
        teamid:   team_id,
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
                            error!("join_team response msg: {:?}", recv.msg);
                        }
                    }
                    else if let Ok(recv) = serde_json::from_str(res_data) {
                        let recv: JavaResponseAlreadyIn = recv;
                        if recv.code == 931 {
                            return Ok(());
                        }
                    }
                    else {
                        error!("join_team - response can't parse: {:?}", res_data);
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