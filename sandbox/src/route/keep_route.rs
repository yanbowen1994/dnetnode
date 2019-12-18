use std::net::IpAddr;
use std::str::FromStr;

use super::*;

pub fn keep_route(local_vip: Option<IpAddr>, new_hosts: Vec<IpAddr>, dev: String) {
    let now_route = parse_routing_table();
    let old_hosts: Vec<IpAddr> = now_route.iter()
        .filter_map(|route| {
            if route.dev == dev {
                if let Ok(ip) = IpAddr::from_str(&route.dst) {
                    if 32 == route.mask {
                        Some(ip)
                    }
                    else {
                        None
                    }
                }
                else {
                    None
                }
            }
            else {
                None
            }
        })
        .collect();

    for host in &new_hosts {
        if !is_in_routing_table(&now_route, host, 32, &dev) {
            add_route(host, 32, &dev);
        }
    }

    for host in old_hosts {
        if Some(host) != local_vip {
            if !new_hosts.contains(&host) {
                if is_in_routing_table(&now_route, &host, 32, &dev) {
                    del_route(&host, 32, &dev);
                }
            }
        }
    }
}
