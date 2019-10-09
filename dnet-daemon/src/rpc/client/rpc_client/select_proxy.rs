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

pub fn select_proxy(connect_to_vec: Vec<ConnectTo>) -> Result<()> {
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
        connect_to = select_min_rtt_proxys(proxy_rtt);
        connect_to_change_restart_tunnel = true;
    }
    else {
        let is_contain = false;
        for local_connect_to in local_connect_to_vec {
            // TODO
            if !connect_to_vec.contains(&local_connect_to) {
                connect_to = select_min_rtt_proxys(proxy_rtt.clone());
                connect_to_change_restart_tunnel = true;
            }
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

    if connect_to_change_restart_tunnel {
        let mut tinc = TincOperator::new();
        tinc.set_info_to_local()
            .map_err(Error::TincOperator)?;
        tinc.restart_tinc()
            .map_err(Error::TincOperator)?;
    }

    Ok(())
}

// Now, connect to only one proxy.
fn select_min_rtt_proxys(proxy_rtt: HashMap<ConnectTo, u32>) -> Vec<ConnectTo> {
    let mut min_rtt = 0;
    let mut connect_to = vec![];

    for (proxy, rtt) in proxy_rtt {
        if min_rtt == 0 || min_rtt > rtt {
            min_rtt = rtt;
            connect_to = vec!(proxy.clone());
        }
    }
    connect_to
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