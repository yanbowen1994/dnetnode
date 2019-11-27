use serde_json::json;

use crate::info::get_info;
use crate::settings::get_settings;
use crate::rpc::http_request::post;
use crate::rpc::Result;

pub(super) fn connect_disconnect_team(
    team_id: &str,
    connect_or_disconnect: bool,
) -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vlan/device/proxy/connectTeam";

    let info = get_info().lock().unwrap();
    let device_serial = info.client_info.device_name.clone();
    std::mem::drop(info);

    let status = if connect_or_disconnect {
        1
    }
    else {
        0
    };

    let data = json!({
        "deviceSerial": device_serial,
        "status": status,
        "teamId": team_id,
    }).to_string();

    let _ = post(&url, &data)?;
    Ok(())
}