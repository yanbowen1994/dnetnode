use std::net::IpAddr;
use std::str::FromStr;

use dnet_types::team::NetSegment;

use super::*;

pub fn keep_route(local_vip: Option<IpAddr>, new_hosts: Vec<NetSegment>, dev: String) {
    let now_route = parse_routing_table();
    let old_hosts: Vec<NetSegment> = now_route.iter()
        .filter_map(|route| {
            if route.dev == dev {
                if let Ok(ip) = IpAddr::from_str(&route.dst) {
                    let net_segment = NetSegment::new(ip, route.mask);
                    Some(net_segment)
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
        if !is_in_routing_table(&now_route, &host.ip, host.mask, &dev) {
            add_route(&host.ip, host.mask, &dev);
        }
    }

    for host in old_hosts {
        if Some(host.ip) != local_vip {
            if !new_hosts.contains(&host) {
                if is_in_routing_table(&now_route, &host.ip, host.mask, &dev) {
                    del_route(&host.ip, host.mask, &dev);
                }
            }
        }
    }
}
