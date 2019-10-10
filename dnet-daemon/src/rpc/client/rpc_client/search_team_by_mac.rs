use std::str::FromStr;

use crate::info::{get_info, get_mut_info};

use super::error::{Error, Result};
use crate::settings::get_settings;
use super::post;
use super::types::DeviceId;
use std::net::IpAddr;
use crate::rpc::client::rpc_client::types::teams::JavaResponseTeamSearch;
use crate::rpc::client::rpc_client::search_team_handle::search_team_handle;

pub(super) fn search_team_by_mac() -> Result<()> {
    let url    = get_settings().common.conductor_url.clone()
        + "/vppn/api/v2/client/searchteambymac";;
    let device_id;
    let cookie;
    {
        let info = get_info().lock().unwrap();
        device_id = get_settings().common.username.clone();
//        device_id = info.client_info.uid.clone();
        cookie = info.client_info.cookie.clone();
    }

    let data = DeviceId {
        deviceid: device_id.clone(),
    }.to_json();

    let mut res = post(&url, &data, &cookie)?;

    if res.status().as_u16() == 200 {
        let res_data = &res.text().map_err(Error::Reqwest)?;
        let recv: JavaResponseTeamSearch = serde_json::from_str(res_data)
            .map_err(|e|{
                error!("search_team_by_mac - response data: {}", res_data);
                Error::ParseJsonStr(e)
            })?;

        if recv.code == Some(200) {
            let mut info = get_mut_info().lock().unwrap();
            if let Some(mut teams) = recv.data {
                search_team_handle(teams)?;
            }
            else {
                return Err(Error::client_not_bound);
            }
            return Ok(());
        }
        else {
            if let Some(msg) = recv.msg {
                if &msg == "Group does not exist" {
                    return Err(Error::client_not_bound);
                }
                return Err(Error::SearchTeamByMac(msg));
            }
        }
    }
    else {
        let mut err_msg = "Unknown reason.".to_string();
        if let Ok(msg) = res.text() {
            err_msg = msg;
        }
        return Err(Error::SearchTeamByMac(
            format!("Code:{} Msg:{}", res.status().as_u16(), err_msg).to_string()));
    }
    return Err(Error::SearchTeamByMac("Unknown reason.".to_string()));
}



