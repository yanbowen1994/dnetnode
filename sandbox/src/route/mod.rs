#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(windows)]
#[path = "windows.rs"]
mod imp;

mod keep_route;
pub mod types;
pub mod error;

use std::net::IpAddr;

pub use keep_route::keep_route;
pub use imp::{
    add_route,
    del_route,
    is_in_routing_table,
    parse_routing_table,
    parse_netmask_to_cidr,
};
use self::types::RouteInfo;
use self::error::{Error, Result};

pub fn get_default_route() -> Result<RouteInfo> {
    let mut route = None;
    let _ = parse_routing_table()?
        .into_iter()
        .map(|route_info| {
            if route_info.dst == "0.0.0.0"
                && route_info.mask == 0 {
                route = Some(route_info)
            }
            else if route_info.dst == "default" {
                route = Some(route_info)
            }
        })
        .collect::<Vec<()>>();
    if let Some(route) = route {
        Ok(route)
    }
    else {
        Err(Error::default_route_not_found)
    }
}

pub fn get_mac(dev: &str) -> Result<String> {
    let mac = if let Ok(mac) = mac_address::mac_address_by_name(dev) {
        if let Some(mac) = mac {
            mac
        }
        else {
            mac_address::get_mac_address()
                .map_err(Error::get_mac_address)?
                .ok_or(Error::get_mac_address_empty)?
        }
    }
    else {
        mac_address::get_mac_address()
            .map_err(Error::get_mac_address)?
            .ok_or(Error::get_mac_address_empty)?
    };

    Ok(mac.to_string())
}

pub fn replace_ip_last_to_zero(ip: &IpAddr) -> Option<String> {
    if ip.is_ipv4() {
        let ip_string = ip.to_string();
        let ip_segment = ip_string.split(".").collect::<Vec<&str>>();
        let new_ip = format!("{}.{}.{}.{}",
                             ip_segment[0],
                             ip_segment[1],
                             ip_segment[2],
                             0);
        Some(new_ip)
    }
    else {
        None
    }
}

#[test]
fn test_get_default_route() {
    let route = get_default_route();
    println!("{:?}", route);
    let mac = get_mac(&route.unwrap().dev);
    println!("{:?}", mac);
}