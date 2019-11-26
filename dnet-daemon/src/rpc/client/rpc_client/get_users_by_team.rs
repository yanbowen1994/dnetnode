use serde_json::Value;

use crate::info::UserInfo;
use crate::settings::get_settings;
use crate::rpc::{Error, Result};
use crate::rpc::http_request::get;

// if return true restart tunnel.
pub(super) fn get_users_by_team(teamid: &str) -> Result<Vec<UserInfo>> {
    let url = get_settings().common.conductor_url.clone()
        + "/vlan/team/user/queryAll?teamId=" + teamid;

    let res_data = get(&url)?;

    let res_data = res_data.get("records")
        .ok_or(Error::ResponseParse(res_data.to_string()))?;

    let recv: Vec<Value> = res_data.as_array()
        .ok_or({
            Error::ResponseParse(res_data.to_string())
        })?.to_owned();

    let mut user_infos: Vec<UserInfo> = vec![];
    for user in recv {
        let name = match user.get("username")
            .and_then(|x| {
                x.as_str()
            })
            .and_then(|x| {
                Some(x.to_owned())
            }) {
            Some(x) => Some(x),
            None => continue
        };
        let email = user.get("useremail")
            .and_then(|x| {
                x.as_str()
            })
            .and_then(|x| {
                Some(x.to_owned())
            });
        let photo = user.get("photo")
            .and_then(|x| {
                x.as_str()
            })
            .and_then(|x| {
                Some(x.to_owned())
            });
        let user_info = UserInfo {
            name,
            email,
            photo,
        };
        user_infos.push(user_info);
    }
    println!("{:?}", user_infos);
    return Ok(user_infos);
}