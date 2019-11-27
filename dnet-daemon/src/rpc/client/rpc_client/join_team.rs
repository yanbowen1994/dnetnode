use serde_json::json;

use crate::info::get_info;
use crate::settings::get_settings;
use crate::rpc::http_request::post;
use crate::rpc::Result;

pub(super) fn join_team(team_id: &str) -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vlan/team/member/addByDeviceSerial";

    let info = get_info().lock().unwrap();
    let device_id = info.client_info.device_name.clone();
    std::mem::drop(info);

    let data = json!({
        "deviceSerials": device_id,
        "teamId": team_id
    }).to_string();

    let _ = post(&url, &data)?;
    Ok(())
}