use crate::settings::{get_settings, Settings};

use super::{Error, Result};
use super::post;

pub(super) fn login() -> Result<()> {
    let settings = get_settings();
    let url = settings.common.conductor_url.clone() + "/login";
    let data = User::new_from_settings(settings).to_json();

    debug!("client_login - request url: {} ",url);
    debug!("client_login - request data:{}",data);

    let mut res = post(&url, &data, "")?;

    debug!("client_login - response code: {}", res.status().as_u16());
    debug!("client_login - response header: {:?}", res.headers());

    if res.status().as_u16() == 200 {
        let res_data = res.text().map_err(Error::Reqwest)?;
        debug!("client_login - response data: {:?}", res_data);
        let _login: Login = serde_json::from_str(&res_data)
            .map_err(|e|{
                error!("client_login - response data: {:?}", res_data);
                Error::LoginParseJsonStr(e)
            })?;

        return Ok(());
    }
    else {
        let mut err_msg = "Unknown reason.".to_string();
        if let Ok(msg) = res.text() {
            err_msg = msg;
        }
        return Err(Error::LoginFailed(
            format!("Code:{} Msg:{}", res.status().as_u16(), err_msg).to_string()));
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
    username: String,
    password: String,
}
impl User {
    fn new_from_settings(settings: &Settings) -> Self {
        User {
            username: settings.common.username.clone(),
            password: settings.common.password.clone(),
        }
    }
    fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Login {
    code:    i32,
    data:    LoginUser,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginUser {
    pub userid:                         String,
    pub username:                       String,
    pub useremail:                      String,
//    pub photo:                          Option<String>,
//    pub devices:                        Option<Vec<Device>>,
    pub enable_autogroup:               bool,
    pub enable_autoothergroup:          bool,
    pub enable_autonetworking:          bool,
    pub invitetime:                     String,
}