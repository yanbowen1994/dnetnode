use std::str::FromStr;
use std::net::IpAddr;

use tinc_plugin::ConnectTo;

use crate::info::get_info;
use crate::tinc_manager::TincOperator;
use crate::settings::get_settings;
use super::post;
use super::{Error, Result};

pub(super) fn client_get_online_proxy() -> Result<Vec<ConnectTo>> {
    let settings = get_settings();
    let url = settings.common.conductor_url.clone()
        + "/vppn/api/v2/proxy/getonlineproxy";

    let cookie;
    {
        cookie = get_info().lock().unwrap().client_info.cookie.clone();
    }
    debug!("client_get_online_proxy - request url: {}", url);

    let mut res = post(&url, "", &cookie)?;

    debug!("client_get_online_proxy - response code: {}", res.status().as_u16());

    if res.status().as_u16() == 200 {
        let res_data = &res.text().map_err(Error::Reqwest)?;
        let recv: GetOnlinePorxyRecv = serde_json::from_str(res_data)
            .map_err(|e|{
                error!("client_get_online_proxy - response data: {}", res_data);
                Error::GetOnlineProxyParseJsonStr(e)
            })?;

        if recv.code == 200 {
            let proxy_vec: Vec<Proxy> = recv.data;

            let mut connect_to_vec = vec![];

            let tinc = TincOperator::new();

            for proxy in proxy_vec {
                if let Ok(proxy_ip) = IpAddr::from_str(&proxy.ip) {
                    if let Ok(proxy_vip) = IpAddr::from_str(&proxy.vip) {
                        let connect_to = ConnectTo::from(proxy_ip, proxy_vip, proxy.pubkey);
                        tinc.set_hosts(
                            true,
                            &connect_to.ip.to_string(),
                            &("Address=".to_string() +
                                &(&connect_to.ip.clone()).to_string() +
                                "\n" +
                                &connect_to.pubkey +
                                "Port=50069")
                        )
                            .map_err(Error::TincOperator)?;

                        connect_to_vec.push(connect_to);
                        continue
                    }
                }
                error!("client_get_online_proxy - One proxy data invalid: {:?}", proxy);
            }

            return Ok(connect_to_vec);
        }
        else {
            return Err(Error::http(recv.code));
        }
    }
    else {
        let mut err_msg = "Unknown reason.".to_string();
        if let Ok(msg) = res.text() {
            err_msg = msg;
        }
        return Err(Error::GetOnlineProxy(
            format!("Code:{} Msg:{}", res.status().as_u16(), err_msg).to_string()));
    }
    return Err(Error::GetOnlineProxy("Unknown reason.".to_string()));
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetOnlinePorxyRecv {
    code:        i32,
    msg:         Option<String>,
    data:        Vec<Proxy>,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Proxy {
    id:                 String,
    ip:                 String,
    country:            Option<String>,
    region:             Option<String>,
    city:               Option<String>,
    username:           Option<String>,
    teamcount:          Option<u32>,
    ispublic:           Option<bool>,
    vip:                String,
    pubkey:             String,
}