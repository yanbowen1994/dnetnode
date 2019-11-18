use serde_json::{json, Value};

use crate::info::{get_info, UserInfo};
use crate::settings::get_settings;
use super::error::{Error, Result};
use super::post;

// if return true restart tunnel.
pub(super) fn get_users_by_team(teamid: &str) -> Result<Vec<UserInfo>> {
    let url    = get_settings().common.conductor_url.clone()
        + "/vppn/api/v2/client/getusersbyteam";
    let cookie;
    {
        let info = get_info().lock().unwrap();
        cookie = info.client_info.cookie.clone();
    }

    let data = json!({"teamid": teamid});

    let mut res = post(&url, &data.to_string(), &cookie)?;

    if res.status().as_u16() == 200 {
        let res_data = &res.text().map_err(Error::Reqwest)?;
        let recv: Value = serde_json::from_str(res_data)
            .map_err(|e|{
                error!("response data: {}", res_data);
                Error::ParseJsonStr(e)
            })?;

        let recv_code = recv.get("code")
            .and_then(|code| {
                code.as_u64()
            })
            .unwrap_or(500);

        if recv_code == 200 {
            println!("{:?}", recv);
            let user_infos = recv.get("data")
                .and_then(|data| {
                    data.as_array()
                })
                .and_then(|data| {
                    let mut user_infos: Vec<UserInfo> = vec![];
                    for user in data {
                        let name = user.get("username")
                            .and_then(|x| {
                                x.as_str()
                            })
                            .and_then(|x| {
                                Some(x.to_owned())
                            });
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
                    Some(user_infos)
                });
            if let Some(user_infos) = user_infos {
                println!("{:?}", user_infos);
                return Ok(user_infos);
            }
        }
        else {
            return Err(Error::http(recv_code as i32))
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