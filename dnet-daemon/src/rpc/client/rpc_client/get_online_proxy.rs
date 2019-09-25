use std::sync::{Arc, Mutex};
use std::str::FromStr;
use std::net::IpAddr;

extern crate tokio_ping;
extern crate tokio_core;
use futures::{Future, Stream, future};
use self::tokio_core::reactor::Core;

use tinc_plugin::ConnectTo;

use crate::info::{Info, get_info, get_mut_info};
use crate::tinc_manager::TincOperator;
use crate::settings::get_settings;
use super::types::DeviceId;
use super::post;
use super::{Error, Result};
use std::time::Duration;

pub(super) fn client_get_online_proxy() -> Result<()> {
    let settings = get_settings();
    let url = settings.common.conductor_url.clone()
        + "/vppn/api/v2/proxy/getonlineproxy";

    let cookie;
    {
        cookie = get_info().lock().unwrap().client_info.cookie.clone();
    }
    debug!("client_get_online_proxy - request url: {}",url);

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
            let mut min_rtt = 0;
            let mut connect_to = vec![];
            for proxy in &connect_to_vec {
                if let Some(rtt) = pinger(proxy.ip) {
                    if min_rtt == 0 || min_rtt > rtt {
                        min_rtt = rtt;
                        connect_to = vec!(proxy.clone());
                    }
                }
            }
            if connect_to.len() == 0 && connect_to_vec.len() > 0 {
                let proxy = connect_to_vec[0].to_owned();
                connect_to = vec![proxy];
            }

            if connect_to.len() == 0 {
                return Err(Error::NoUsableProxy);
            }
            {
                let mut info = get_mut_info().lock().unwrap();
                info.tinc_info.connect_to = connect_to;
            }
            return Ok(());
        }
        else {
            if let Some(msg) = recv.msg {
                return Err(Error::GetOnlineProxy(msg));
            }
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

fn pinger(addr: IpAddr) -> Option<u32> {
    let mut reactor = Core::new().unwrap();
    let timeout = Duration::from_secs(1);
    let pinger = tokio_ping::Pinger::new();
    let stream = pinger
        .and_then(move |pinger| Ok(pinger.chain(addr).timeout(timeout).stream()));
    let future = stream.and_then(|stream| {
        stream.take(3)
            .fold(Vec::new(), |mut acc, result|{
                acc.push(result);
                future::ok::<Vec<Option<Duration>>, tokio_ping::Error>(acc)
            })
    });

    if let Ok(mut rtts) = reactor.run(future) {
        let mut num = 0;
        let mut sum = 0;
        for rtt in rtts {
            if let Some(rtt_duration) = rtt {
                num += 1;
                sum += rtt_duration.as_millis();
            }
        }

        let average: Option<u32> = match num {
            0 => None,
            _ => Some(sum as u32 / num),
        };
        return average;
    }
    None
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