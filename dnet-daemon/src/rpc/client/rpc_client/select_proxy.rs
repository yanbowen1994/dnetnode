use std::net::IpAddr;
use std::time::Duration;
use std::collections::HashMap;

extern crate pinger;

use tinc_plugin::ConnectTo;

use crate::info::{get_mut_info, get_info};
use crate::rpc::{Error, Result};

pub fn select_proxy(connect_to_vec: Vec<ConnectTo>) -> Result<bool> {
    let mut connect_to_change_restart_tunnel = false;

    let mut min_rtt = 0;
    let mut proxy_rtt = HashMap::new();
    for proxy in &connect_to_vec {
        if let Some(rtt) = ping(proxy.ip) {
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
        return Err(Error::http(511));
    }

    let mut info = get_mut_info().lock().unwrap();
    info.tinc_info.connect_to = connect_to;
    std::mem::drop(info);

    Ok(connect_to_change_restart_tunnel)
}

// Now, connect to only one proxy.
fn select_min_rtt_proxys(proxy_rtt: HashMap<ConnectTo, u128>) -> (Vec<ConnectTo>, u128) {
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

fn ping(addr: IpAddr) -> Option<u128> {
    let timeout = Some(Duration::from_secs(1));
    let ttl = None;
    let ident = None;
    let seq_cnt = Some(1);
    let payload = None;
    if let Ok(rtt) = pinger::ping(addr, timeout, ttl, ident, seq_cnt, payload) {
        return Some(rtt);
    }
    else {
        return None;
    }
}