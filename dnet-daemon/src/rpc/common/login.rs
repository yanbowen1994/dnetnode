use crate::settings::get_settings;
use crate::rpc::http_request::post;
#[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
use crate::info::UserInfo;
use crate::info::get_mut_info;
#[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
use crate::info::get_info;
use crate::rpc::{Error, Result};

pub fn login() -> Result<()> {
    let settings = get_settings();
    #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
        {
            let url = settings.common.conductor_url.clone() + "/vlan/login";
            let data = create_request();
            let res = post(&url, &data)?;
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
        }
    #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
        {
            let url = settings.common.conductor_url.clone() + "/vlan/router/login";
            let data = create_request();
            let res = post(&url, &data)?;
            info!("result: {:?}", res);

            let token = res.get("token")
                .and_then(|token| {
                    token.as_str()
                })
                .ok_or(Error::ResponseParse(res.to_string()))?;

            let mut info = get_mut_info().lock().unwrap();
            info.node.token = token.to_owned();
        }
    Ok(())
}

fn create_request() -> String {
    #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
        {
            let info = get_info().lock().unwrap();
            let username = info.client_info.device_name.clone();
            let password = info.client_info.device_password.clone();
            serde_json::json!({
                "deviceSerial": username,
                "password": password,
            }).to_string()
        }

    #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
        {
            let settings = get_settings();
            let username = settings.common.username.clone();
            let password = settings.common.password.clone();
            serde_json::json!({
                "username": username,
                "password": password,
            }).to_string()
        }
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