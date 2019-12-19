#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(windows)]
#[path = "windows.rs"]
mod imp;

mod keep_route;
pub use keep_route::keep_route;
pub mod types;


pub use imp::{
    add_route,
    del_route,
    is_in_routing_table,
    parse_routing_table,
    parse_netmask_to_cidr,
};
use crate::route::types::RouteInfo;

pub fn get_default_route() -> Option<RouteInfo> {
    let mut route = None;
    let _ = parse_routing_table()
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
    route
}

pub fn get_mac(dev: &str) -> Option<String> {
    let get_frist_dev_mac = || mac_address::get_mac_address()
        .ok()
        .and_then(|x|x);

    let mac = mac_address::mac_address_by_name(dev)
        .ok()
        .unwrap_or(get_frist_dev_mac())
        .unwrap_or(get_frist_dev_mac()?);
    Some(mac.to_string())
}


#[test]
fn test_get_default_route() {
    let route = get_default_route();
    println!("{:?}", route);
    let mac = get_mac(&route.unwrap().dev);
    println!("{:?}", mac);
}