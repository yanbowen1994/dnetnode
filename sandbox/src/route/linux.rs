use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::process::Command;

use super::types::RouteInfo;
use super::replace_ip_last_to_zero;
use super::error::{Error, Result};

// netmask CIDR
pub fn add_route(ip: &IpAddr, netmask: u32, dev: Option<String>, gw: Option<IpAddr>) {
    if let Some(gw) = gw {
        if let Some(ip) = replace_ip_last_to_zero(ip) {
            let ip_mask = ip + "/" + &format!("{}", netmask);
            let gw = gw.to_string();
            let res = Command::new("route")
                .args(vec!["add", "-net", &ip_mask, "gw", &gw])
                .output();
            info!("add_route {} gw {:?} res {:?}", ip_mask, gw, res);
        }
        else {
            warn!("add_route ipv6 not support")
        }
    }
    else if let Some(dev) = dev {
        let ip_mask = ip.clone().to_string() + "/" + &format!("{}", netmask);
        let _ = Command::new("ip")
            .args(vec!["route", "add", &ip_mask, "dev", &dev])
            .output();
    }
}

pub fn del_route(ip: &IpAddr, netmask: u32, dev: &str) {
    let ip_mask = ip.clone().to_string() + "/" + &format!("{}", netmask);
    let _ = Command::new("ip")
        .args(vec!["route", "del", &ip_mask, "dev", dev])
        .output();
}

pub fn is_in_routing_table(routing_table: &Vec<RouteInfo>,
                           ip: &IpAddr,
                           netmask: u32,
                           dev: &str
) -> bool {
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

#[cfg(not(target_arch = "arm"))]
pub fn parse_routing_table() -> Result<Vec<RouteInfo>> {
    let output = Command::new("ip")
        .args(vec!["route"])
        .output()
        .map_err(Error::exec_ip_route_cmd)?
        .stdout;
    let output = String::from_utf8(output)
        .map_err(|_|Error::parse_ip_route_cmd)?;

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
    Ok(route_infos)
}

#[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
pub fn parse_routing_table() -> Result<Vec<RouteInfo>> {
    let output = Command::new("route")
        .output()
        .map_err(Error::exec_route_cmd)?
        .stdout;

    let tables = String::from_utf8(output)
        .map_err(|_|Error::parse_route_cmd)?
        .split("\n")
        .collect::<Vec<&str>>()
        .iter()
        .filter_map(|bar| {
            if bar.contains("Destination") {
                return None;
            }
            let mut bar = bar.as_bytes().to_vec();
            bar.dedup_by(|a, b|a == b && *a == 32);
            let bar = match String::from_utf8(bar) {
                Ok(x) => x,
                Err(_) => return None,
            };

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
        .collect::<Vec<RouteInfo>>();
    Ok(tables)
}

#[cfg(test)]
mod test {
    use std::net::IpAddr;
    use std::str::FromStr;
    use crate::route::{add_route, del_route};
    use crate::route::imp::parse_routing_table;

    #[test]
    fn test() {
        let ip = IpAddr::from_str("12.12.12.12").unwrap();
        add_route(&ip, 32, Some("enp3s0".to_string()), None);

        let stdout = duct::cmd!("route").stdout_capture().run().unwrap().stdout;
        let stdout = String::from_utf8(stdout).unwrap();
        assert!(stdout.contains("12.12.12.12"));
        del_route(&ip, 32, "enp3s0");

        let stdout = duct::cmd!("route").stdout_capture().run().unwrap().stdout;
        let stdout = String::from_utf8(stdout).unwrap();
        assert!(!stdout.contains("12.12.12.12"));

        parse_routing_table();
    }

    #[test]
    fn test_parse_routing_table() {
        println!("{:#?}", parse_routing_table());
    }
}