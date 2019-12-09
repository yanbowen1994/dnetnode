use crate::settings::get_settings;

use crate::rpc::Result;
use crate::rpc::http_request::post;
use tinc_plugin::TincTools;
use dnet_types::tinc_host_status_change::HostStatusChange;

pub fn center_update_tinc_status(change: HostStatusChange) -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vlan/device/proxy/updateTincstatus";

    let (status, vip) = match change {
        HostStatusChange::HostUp(host) => {
            let vip = match TincTools::get_vip_by_filename(&host) {
                Some(x) => x.to_string(),
                None => return Ok(()),
            };
            (1, vip)
        }
        HostStatusChange::HostDown(host) => {
            let vip = match TincTools::get_vip_by_filename(&host) {
                Some(x) => x.to_string(),
                None => return Ok(()),
            };
            (0, vip)
        }
        _ => return Ok(()),
    };

    let data = serde_json::json!({
	    "status": status,
	    "vip": vip,
    }).to_string();
    info!("update_tinc_status: {:?}", data);
    let _ = post(&url, &data)?;
    Ok(())
}