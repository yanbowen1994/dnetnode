use serde_json::json;

use crate::settings::get_settings;
use crate::rpc::http_request::post;
use crate::info::{UserInfo, get_mut_info};
use crate::rpc::{Error, Result};

pub fn login() -> Result<()> {
    let settings = get_settings();
    let url = settings.common.conductor_url.clone() + "/vlan/login";

    let username = settings.common.username.clone();
    let password = settings.common.password.clone();

    let data: serde_json::Value = json!({"username": username, "password": password});

    let res = post(&url, &data.to_string())?;
    info!("result: {:?}", res);

    let token = res.get("token")
        .and_then(|token| {
            token.as_str()
        })
        .ok_or(Error::ResponseParse(res.to_string()))?;

    let body: LoginResponse = serde_json::from_value(res.clone())
        .map_err(|_|Error::ResponseParse(res.to_string()))?;

    let user_info = UserInfo {
        name:       body.username,
        email:      body.email,
        photo:      body.avatar,
    };

    let mut info = get_mut_info().lock().unwrap();
    info.node.token = token.to_owned();
    info.user = user_info;
    Ok(())
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct LoginResponse {
    avatar:      Option<String>,
    birthday:    Option<String>,
    companyId:   Option<String>,
    createTime:  Option<String>,
    delFlag:     Option<i32>,
    email:       Option<String>,
    gender:      Option<i32>,
    id:          Option<String>,
    loginType:   Option<i32>,
    password:    Option<String>,
    phone:       Option<String>,
    realname:    Option<String>,
    salt:        Option<String>,
    status:      Option<i32>,
    updateBy:    Option<String>,
    updateTime:  Option<String>,
    username:    Option<String>,
}