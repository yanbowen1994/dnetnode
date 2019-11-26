use serde_json;

use crate::settings::get_settings;

use crate::rpc::Result;
use crate::rpc::http_request::post;
use crate::info::get_mut_info;
use crate::rpc::types::JavaProxy;
use crate::rpc::Error;

pub fn proxy_add() -> Result<()> {
    let url = get_settings().common.conductor_url.clone() + "/vlan/proxy/add";
    let data = JavaProxy::new().to_json();

    debug!("Request {}", data);
    let res_data = post(&url, &data.to_string())?;
    let res_proxy: JavaProxy = serde_json::from_value(res_data.clone())
        .map_err(|_|Error::ResponseParse(res_data.to_string()))?;
    info!("Response {:?}", res_proxy);
    let proxy = res_proxy.clone().parse_to_proxy_info()
        .ok_or(Error::ResponseParse(res_proxy.to_json()))?;
    let mut info = get_mut_info().lock().unwrap();
    let tmp = info.proxy_info.auth_id.clone();
    info.proxy_info = proxy.clone();
    info.proxy_info.auth_id = tmp;
    info.tinc_info.vip = Some(proxy.vip);
    Ok(())
}