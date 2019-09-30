use dnet_types::tinc_host_status_change::HostStatusChange;
use crate::settings::get_settings;
use crate::info::get_info;
use super::post::post;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct HereRequest {
    name:       String,
    state:      u32,
    proxyIp:    String,
}

impl HereRequest {
    fn new(name:       String,
           status:     u32,
           proxy_ip:    String,
    ) -> Self {
        Self {
            name,
            state: status,
            proxyIp: proxy_ip
        }
    }

    fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}

pub fn report_host_status_change(host_status_change: HostStatusChange) {
    let url = get_settings().common.conductor_url.clone()
        + "/vppn/api/v2/client/statusNotify";

    let info = get_info().lock().unwrap();
    let cookie = info.proxy_info.cookie.clone();
    let proxy_ip = match info.proxy_info.ip.clone() {
        Some(x) => x,
        _ => return
    };
    std::mem::drop(info);

    let (name, status) = match host_status_change {
        HostStatusChange::HostUp(name) => (name, 1.to_owned()),
        HostStatusChange::HostDown(name) => (name, 0.to_owned()),
        _ => return,
    };

    let data = HereRequest::new(name, status, proxy_ip.to_string()).to_json();

    debug!("report_host_status_change - request url: {}", url);
    debug!("report_host_status_change - request data: {}", data);

    let mut res = match post(&url, &data, &cookie)
        .map_err(|e|error!("{:?}", e)) {
        Ok(x) => x,
        _ => return
    };

    debug!("report_host_status_change - response code: {}", res.status().as_u16());

    if res.status().as_u16() == 200 {
        return;
    }

    let _ = res.text()
        .map(|res_data|
                 error!("report_host_status_change - response: {}", res_data)
        )
        .map_err(|e| {
            error!("{}", format!("report_host_status_change {:?}", e));
        });
    return;
}