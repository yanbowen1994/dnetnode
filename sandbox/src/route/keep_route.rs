use std::net::IpAddr;
//use std::str::FromStr;

use dnet_types::team::NetSegment;

use super::*;

pub fn keep_route(_local_vip: Option<IpAddr>, new_hosts: Vec<NetSegment>, dev: String) {
    let now_route = match parse_routing_table() {
        Some(x) => x,
        None => {
            error!("parse_routing_table");
            return;
        },
    };
//    let old_hosts: Vec<NetSegment> = now_route.iter()
//        .filter_map(|route| {
//            if route.dev == dev {
//                if let Ok(ip) = IpAddr::from_str(&route.dst) {
//                    let gw = if let Ok(gw) = IpAddr::from_str(&route.gw) {
//                        Some(gw)
//                    }
//                    else {
//                        None
//                    };
//
//                    let net_segment = NetSegment::new(ip, route.mask, gw);
//                    Some(net_segment)
//                }
//                else {
//                    None
//                }
//            }
//            else {
//                None
//            }
//        })
//        .collect();

    for host in &new_hosts {
        if !is_in_routing_table(&now_route, &host.ip, host.mask, &dev) {
            if host.mask == 32 {
                add_route(&host.ip, host.mask, Some(dev.clone()), None);
            }
            else {
                add_route(&host.ip, host.mask, None, host.gw);
            }
        }
    }

//    for host in old_hosts {
//        if Some(host.ip) != local_vip {
//            if !new_hosts.contains(&host) {
//                if is_in_routing_table(&now_route, &host.ip, host.mask, &dev) {
//                    del_route(&host.ip, host.mask, &dev);
//                }
//            }
//        }
//    }
}
