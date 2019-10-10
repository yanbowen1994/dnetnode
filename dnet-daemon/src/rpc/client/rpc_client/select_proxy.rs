use std::net::IpAddr;
use std::time::Duration;
use std::collections::HashMap;

use futures::{Future, Stream, future};
extern crate tokio_ping;
extern crate tokio_core;
use self::tokio_core::reactor::Core;
use tinc_plugin::ConnectTo;

use crate::tinc_manager::TincOperator;
use crate::info::{get_mut_info, get_info};
use super::{Error, Result};

pub fn select_proxy(connect_to_vec: Vec<ConnectTo>) -> Result<bool> {
    let mut connect_to_change_restart_tunnel = false;

    let mut min_rtt = 0;
    let mut proxy_rtt = HashMap::new();
    for proxy in &connect_to_vec {
        if let Some(rtt) = pinger(proxy.ip) {
            if min_rtt == 0 || min_rtt > rtt {
                min_rtt = rtt;
                proxy_rtt.insert(proxy.clone(), rtt);
            }
        }
    }

    let info = get_info().lock().unwrap();
    let local_connect_to_vec = info.tinc_info.connect_to.clone();
    std::mem::drop(info);

    let mut connect_to = vec![];
    if local_connect_to_vec.len() == 0 {
        let (connect_to_tmp, _) = select_min_rtt_proxys(proxy_rtt);
        connect_to = connect_to_tmp;
        connect_to_change_restart_tunnel = true;
    }
    else {
        let mut need_change_proxy = true;
        let (min_rtt_proxy, min_rtt) = select_min_rtt_proxys(proxy_rtt.clone());
        for local_connect_to in local_connect_to_vec {
            // proxy offline
            if connect_to_vec.contains(&local_connect_to) {
                if let Some(rtt) = proxy_rtt.get(&local_connect_to).cloned() {
                    // if (min_rtt as f64 / rtt as f64) < 0.7 && rtt > 100
                    //     Bad proxy network need change proxy.
                    if !((min_rtt as f64 / rtt as f64) < 0.7 && rtt > 100) {
                        need_change_proxy = false;
                    }
                }
            }
        }
        if need_change_proxy {
            connect_to = min_rtt_proxy;
            connect_to_change_restart_tunnel = true;
        }
    }

    if connect_to.len() == 0 && connect_to_vec.len() > 0 {
        let proxy = connect_to_vec[0].to_owned();
        connect_to = vec![proxy];
    }

    if connect_to.len() == 0 {
        return Err(Error::no_usable_proxy);
    }

    let mut info = get_mut_info().lock().unwrap();
    info.tinc_info.connect_to = connect_to;
    std::mem::drop(info);

    Ok(connect_to_change_restart_tunnel)
}

// Now, connect to only one proxy.
fn select_min_rtt_proxys(proxy_rtt: HashMap<ConnectTo, u32>) -> (Vec<ConnectTo>, u32) {
    let mut min_rtt = 0;
    let mut connect_to = vec![];

    for (proxy, rtt) in proxy_rtt {
        if min_rtt == 0 || min_rtt > rtt {
            min_rtt = rtt;
            connect_to = vec!(proxy.clone());
        }
    }
    (connect_to, min_rtt)
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

    if let Ok(rtts) = reactor.run(future) {
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