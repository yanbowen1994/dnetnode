use serde_json::json;

use crate::info::get_info;
use crate::settings::get_settings;
use crate::rpc::http_request::post;
use crate::rpc::Result;

pub(super) fn out_team(team_id: &str) -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vlan/team/member/deleteBatch";

    let info = get_info().lock().unwrap();
    let device_id = info.client_info.uid.clone();
    let data = json!({
        "teamId":    team_id.to_owned(),
        "deviceIds": vec![device_id],
    }).to_string();

    let _ = post(&url, &data)?;
    Ok(())
}