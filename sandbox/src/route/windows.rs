use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::process::Command;
use crate::route::types::RouteInfo;
extern crate ipconfig;


// netmask CIDR
pub fn add_route(ip: &IpAddr, netmask: u32, dev: &str) {
    let mask = parse_netmask_from_cidr(netmask).to_string();
    let idx_if = get_vnic_index(dev);
    println!("{}", idx_if);
    println!("{}", mask);
    let res = Command::new("route")
        .args(vec!["add", &ip.clone().to_string(), "mask", &mask, &(ip.clone().to_string()), "if", &format!("{}", idx_if)])
        .spawn();
    if let Ok(mut res) = res {
        let _ = res.wait();
    }
}

pub fn del_route(ip: &IpAddr, netmask: u32, _dev: &str) {
    let mask = parse_netmask_from_cidr(netmask).to_string();
    let res = Command::new("ip")
        .args(vec!["route", "delete", &ip.clone().to_string(), "mask",&mask])
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

#[cfg(not(target_arch = "arm"))]
pub fn parse_routing_table() -> Vec<RouteInfo> {
    let output = String::from_utf8(Command::new("ip")
            .args(vec!["route"])
            .output()
            .unwrap().stdout)
        .unwrap();
    let route_infos = output
        .split("\n")
        .collect::<Vec<&str>>()
        .iter()
        .filter_map(|bar| {
            let segments: Vec<&str> = bar
                .split(" ")
                .collect::<Vec<&str>>();
            if segments.len() >= 3 {
                let mut route_info = RouteInfo::empty();
                let tmp = segments[0].split("/").collect::<Vec<&str>>();
                if tmp.len() > 1 {
                    route_info.dst = tmp[0].to_owned();
                    route_info.mask = tmp[1].to_owned().parse().unwrap_or(32);
                } else {
                    route_info.dst = segments[0].to_owned();
                    if segments[0] == "default" {
                        route_info.mask = 0;
                    } else {
                        route_info.mask = 32;
                    }
                }
                for i in 1..((segments.len() as i32) / 2 + 1) as usize {
                    match segments[i * 2 - 1] {
                        "via" => route_info.gw = segments[i * 2].to_owned(),
                        "dev" => route_info.dev = segments[i * 2].to_owned(),
                        "metric" => route_info.metric = segments[i * 2].to_owned()
                            .parse()
                            .unwrap_or(0),
                        _ => (),
                    }
                }

                Some(route_info)
            } else {
                None
            }
        })
        .collect::<Vec<RouteInfo>>();
    route_infos
}

#[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
pub fn parse_routing_table() -> Vec<RouteInfo> {
    String::from_utf8(Command::new("route").output().unwrap().stdout)
        .unwrap()
        .split("\n")
        .collect::<Vec<&str>>()
        .iter()
        .filter_map(|bar| {
            if bar.contains("Destination") {
                return None;
            }
            let mut bar = bar.as_bytes().to_vec();
            bar.dedup_by(|a, b|a == b && *a == 32);
            let bar = String::from_utf8(bar)
                .unwrap();

            let segments = bar
                .split(" ")
                .collect::<Vec<&str>>();

            if segments.len() < 8 {
                return None;
            }

            Some(RouteInfo {
                dst:      segments[0].to_owned(),
                gw:       segments[1].to_owned(),
                mask:     segments[2].parse().unwrap_or(32),
                flags:    segments[3].to_owned(),
                metric:   segments[4].parse().unwrap_or(0),
                ref_:     segments[5].to_owned(),
                use_:     segments[6].to_owned(),
                dev:      segments[7].to_owned(),
            })
        })
        .collect::<Vec<RouteInfo>>()
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
