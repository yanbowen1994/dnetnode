use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::process::Command;
use crate::route::types::RouteInfo;
extern crate ipconfig;


// netmask CIDR
pub fn add_route(ip: &IpAddr, netmask: u32, dev: &str) {
    let mask = parse_netmask_from_cidr(netmask).to_string();
    let idx_if = get_vnic_index(dev);
    let res = Command::new("route")
        .args(vec!["add", &ip.clone().to_string(), "mask", &mask, &(ip.clone().to_string()), "if", &format!("{}", idx_if)])
        .spawn();
    if let Ok(mut res) = res {
        let _ = res.wait();
    }
}

pub fn del_route(ip: &IpAddr, netmask: u32, _dev: &str) {
    let mask = parse_netmask_from_cidr(netmask).to_string();
    let res = Command::new("route")
        .args(vec!["delete", &ip.clone().to_string(), "mask",&mask])
        .spawn();
    if let Ok(mut res) = res {
        let _ = res.wait();
    }
}

pub fn is_in_routing_table(routing_table: &Vec<RouteInfo>, ip: &IpAddr, netmask: u32, dev: &str) -> bool {
    for route_info in routing_table {
//      Skip default route,
        if let Ok(cur_ip) = IpAddr::from_str(&route_info.dst) {
            if route_info.mask == netmask
                && &cur_ip == ip
                && &route_info.dev == dev {
                return true;
            }
        }
    }
    false
}

// CIDR classless inter-domain routing
pub fn parse_netmask_to_cidr(netmask: &str) -> Option<u32> {
    let mut cidr: u32 = 32;
    if let Ok(a) = Ipv4Addr::from_str(netmask) {
        let a = u32::from(a);
        let mut b = 4294967295 as u32 - a;
        loop {
            if b == 0 {
                break;
            }
            b = b >> 1;
            cidr -= 1;
        }
        return Some(cidr);
    }
    None
}

pub fn parse_netmask_from_cidr(netmask: u32) -> IpAddr {
    if netmask == 0 {
        return IpAddr::from_str("0.0.0.0").unwrap();
    }
    let d: u32 = 4294967295 - (1 << (32 - netmask)) + 1;
    let mask = Ipv4Addr::from(d);
    IpAddr::from(mask)
}

#[test]
fn test() {
    let ip = IpAddr::from_str("12.12.12.12").unwrap();
    add_route(&ip, 32, "本地连接");

    del_route(&ip, 32, "本地连接");
    parse_routing_table();
}


#[cfg(windows)]
fn get_vnic_index(dev: &str) -> u32 {
    let adapters = ipconfig::get_adapters().unwrap();
    for interface in adapters {
        if interface.friendly_name() == dev {
            return interface.ipv6_if_index();
        }
    }
    0
}
