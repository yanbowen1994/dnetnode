use crate::settings::get_settings;

use crate::rpc::{Error, Result};
use crate::rpc::http_request::post;
use tinc_plugin::{TincTeam, PID_FILENAME};
use std::collections::HashMap;
use std::net::IpAddr;

pub fn center_get_team_info() -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vlan/team/member/getAllTeammembersVlanTagging";
    let res_data = post(&url, "")?;

    let tinc_team_add = serde_json::from_value::<HashMap<String, Vec<IpAddr>>>(res_data.clone())
        .map_err(|e|{
            error!("response: {:?}", e);
            Error::ResponseParse(res_data.to_string())
        })?;

    let tinc_team = TincTeam {
        add: tinc_team_add,
        delete: HashMap::new(),
    };
    let tinc_pid = get_settings().common.home_path
        .join("tinc").join(PID_FILENAME)
        .to_str().unwrap().to_string();

    if let Err(tinc_team) = tinc_team.send_to_tinc(&tinc_pid) {
        error!("Send team info failed {:?}", tinc_team);
    }

    return Ok(())
}