use crate::info::UserInfo;
use crate::settings::get_settings;
use crate::rpc::Result;
use crate::rpc::http_request::{get, MAX_PAGE, PAGESIZE, get_records};
use serde_json::Value;

// if return true restart tunnel.
pub(super) fn get_users_by_team(teamid: &str) -> Result<Vec<UserInfo>> {
    let mut url = get_settings().common.conductor_url.clone()
        + "/vlan/team/user/queryAll?teamId=" + teamid;

    let mut user_infos: Vec<UserInfo> = vec![];

    for i in 0..MAX_PAGE {
        url = url + &format!("&pageNum={}&pageSize={}", i, PAGESIZE);
        let recv = get(&url)?;
        let recv = get_records(&url, recv)?;

        if recv.len() < PAGESIZE {
            user_infos.append(&mut parse_to_user(recv));
            break
        }
        else {
            user_infos.append(&mut parse_to_user(recv));
        }
    }

    info!("{:?}", user_infos);
    return Ok(user_infos);
}

fn parse_to_user(recv: Vec<Value>) -> Vec<UserInfo> {
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
    user_infos
}