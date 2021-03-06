use crate::settings::get_settings;

use crate::rpc::http_request::loop_post;
use crate::rpc::proxy::types::JavaProxy;
use crate::rpc::Result;

pub fn proxy_heartbeat() -> Result<()> {
    let url = get_settings().common.conductor_url.clone()
        + "/vlan/proxy/heartbeat";

    let data = JavaProxy::new().to_json();

    info!("Request: {}", data);

    let _ = loop_post(&url, &data)?;

    Ok(())
}