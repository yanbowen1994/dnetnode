use crate::settings::get_settings;
use crate::rpc::client::rpc_client::search_team_by_mac::{search_team_inner};
use crate::rpc::{Result};

pub(super) fn search_team_by_user() -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vlan/team/queryMyAll";

    search_team_inner(url, false)?;

    Ok(())
}