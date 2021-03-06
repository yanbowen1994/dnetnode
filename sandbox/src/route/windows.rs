use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::process::Command;
extern crate ipconfig;
use ipconfig::Adapter;

use crate::route::types::RouteInfo;

static mut INDEX_IF: Option<(u32, String)> = None;

// netmask CIDR
pub fn add_route(ip: &IpAddr, netmask: u32, dev: &str) {
    let mask = parse_netmask_from_cidr(netmask).to_string();
    let index_if = match get_index_if(dev) {
        Some(x) => x,
        None => {
            warn!("dev {} not found in system", dev);
            return;
        }
    };
    let res = Command::new("route")
        .args(vec!["add", &ip.clone().to_string(), "mask", &mask, &(ip.clone().to_string()), "if", &format!("{}", index_if)])
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

pub fn parse_routing_table() -> Option<Vec<RouteInfo>> {
    let mut adapters = Adapters::new();
    adapters.load_adapters()?;

    let mut route_info = vec![];

    let output = Command::new("wmic")
        .args(vec!["path", "Win32_IP4RouteTable", "get", "Destination,Mask,InterfaceIndex", "/value"])
        .output()
        .ok()?;
    let output = String::from_utf8(output.stdout).ok()?;

    let lines: Vec<&str> = output.split("\r\r\n\r\r\n\r\r\n")
        .collect::<Vec<&str>>();
    for line in lines {
        let segments: Vec<&str> = line.split("\r\r\n")
            .collect::<Vec<&str>>()
            .into_iter()
            .filter_map(|seg|{
                if seg == "" {
                    None
                }
                else {
                    Some(seg)
                }
            })
            .collect::<Vec<&str>>();
        if segments.len() == 3 {
            let mut dst = None;
            let mut dev = None;
            let mut mask = None;
            for seg in segments {
                if seg.contains("Destination=") {
                    dst = Some(seg.replace("Destination=", ""));
                }
                else if seg.contains("InterfaceIndex=") {
                    dev = match seg.replace("InterfaceIndex=", "")
                        .parse::<u32>()
                        .ok()
                        .and_then(|index| {
                            adapters.get_vnic_dev(index)
                        }) {
                        Some(dev) => Some(dev),
                        None => break,
                    };
                }
                else if seg.contains("Mask=") {
                    mask = match parse_netmask_to_cidr(
                        &seg.replace("Mask=", "")) {
                        Some(x) => Some(x),
                        None => break,
                    };
                }
            }

            if let Some(dst) = dst {
                if let Some(dev) = dev {
                    if let Some(mask) = mask {
                        let route = RouteInfo {
                            dst,
                            gw:         String::new(),
                            mask,
                            flags:      String::new(),
                            metric:     0,
                            ref_:       String::new(),
                            use_:       String::new(),
                            dev,
                        };
                        route_info.push(route);
                    }
                }
            }
        }
    }

    Some(route_info)
}

fn get_index_if(dev: &str) -> Option<u32> {
    unsafe {
        if let Some((index_if, save_dev)) = &INDEX_IF {
            if save_dev == dev {
                return Some(*index_if);
            }
        }
        let mut adapters = Adapters::new();
        adapters.load_adapters()?;
        let index_if = adapters.get_vnic_index(dev);
        INDEX_IF = Some((index_if, dev.to_string()));

        Some(index_if)
    }
}

struct Adapters {
    adapters:   Vec<Adapter>,
}

impl Adapters {
    fn new() -> Self {
        Self {
            adapters: vec![],
        }
    }

    fn load_adapters(&mut self) -> Option<()> {
        let adapters = ipconfig::get_adapters()
            .map_err(|e| {
                error!("{:?}", e);
            })
            .ok()?;
        self.adapters = adapters;
        Some(())
    }

    fn get_vnic_dev(&self, index: u32) -> Option<String> {
        for interface in &self.adapters {
            if interface.ipv6_if_index() == index {
                return Some(interface.friendly_name().to_string());
            }
        }
        None
    }

    fn get_vnic_index(&self, dev: &str) -> u32 {
        for interface in &self.adapters {
            if interface.friendly_name() == dev {
                return interface.ipv6_if_index();
            }
        }
        0
    }
}

#[test]
fn test() {
    let ip = IpAddr::from_str("12.12.12.12").unwrap();
    add_route(&ip, 32, "本地连接");
    del_route(&ip, 32, "本地连接");
    parse_routing_table();
}