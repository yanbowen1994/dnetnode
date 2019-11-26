use crate::settings::get_settings;
use crate::rpc::http_request::post;
use crate::rpc::Result;

// if return true restart tunnel.
pub fn search_team_by_mac() -> Result<bool> {
    let url    = get_settings().common.conductor_url.clone()
        + "/vppn/api/v2/client/searchteambymac";

    let data = String::new();

    let _ = post(&url, &data)?;

    Ok(true)
}



