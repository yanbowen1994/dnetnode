use std::str::FromStr;

use crate::info::{get_info, get_mut_info};

use super::error::{Error, Result};
use crate::settings::get_settings;
use super::post;
use std::net::IpAddr;
use crate::rpc::client::rpc_client::types::teams::JavaResponseTeamSearch;
use crate::rpc::client::rpc_client::search_team_handle::search_team_handle;

// if return true restart tunnel.
pub(super) fn search_user_team() -> Result<bool> {
    let url = get_settings().common.conductor_url.clone()
        + "/vppn/api/v2/client/searchuserteam";
    let cookie;
    {
        let info = get_info().lock().unwrap();
        cookie = info.client_info.cookie.clone();
    }

    let data = RequestStatus {
        status: 0,
    }.to_json();

    let mut res = post(&url, &data, &cookie)?;

    if res.status().as_u16() == 200 {
        let res_data = &res.text().map_err(Error::Reqwest)?;
        let recv: JavaResponseTeamSearch = serde_json::from_str(res_data)
            .map_err(|e|{
                error!("search_user_team - response data: {}", res_data);
                Error::ParseJsonStr(e)
            })?;

        if recv.code == Some(200) {
            if let Some(teams) = recv.data {
                let restart = search_team_handle(teams)?;
                return Ok(restart);
            }
            else {
                warn!("No team in current user.");
                return Err(Error::no_team_in_search_condition);
            }
        }
        else {
            if let Some(msg) = recv.msg {
                return Err(Error::SearchUserTeam(msg));
            }
        }
    }
    else {
        let mut err_msg = "Unknown reason.".to_string();
        if let Ok(msg) = res.text() {
            err_msg = msg;
        }
        return Err(Error::SearchUserTeam(
            format!("Code:{} Msg:{}", res.status().as_u16(), err_msg).to_string()));
    }
    return Err(Error::SearchUserTeam("Unknown reason.".to_string()));
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RequestStatus {
    status: u32,
}
impl RequestStatus {
    fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}