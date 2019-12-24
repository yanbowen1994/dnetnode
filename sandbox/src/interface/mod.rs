pub mod error;
use error::{Error, Result};

use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use dnet_types::team::NetSegment;
use crate::route::get_default_route;
#[cfg(target_arch = "arm")]
use crate::route::parse_routing_table;

pub fn get_default_interface() -> Result<NetSegment> {
    let default_route = get_default_route()
        .map_err(Error::route_table)?;
    info!("{:?}", default_route);
    let ip = match IpAddr::from_str(&default_route.dst) {
        Ok(x) => x,
        Err(_) => {
            if default_route.dst == "default" {
                IpAddr::from(Ipv4Addr::new(0, 0, 0, 0))
            }
            else {
                return Err(Error::default_interface_not_found);
            }
        }
    };
    let gw = IpAddr::from_str(&default_route.gw)
        .ok();

    Ok(NetSegment {
        ip,
        mask: default_route.mask,
        gw,
    })
}

#[cfg(target_arch = "arm")]
pub fn get_lans(if_name: Vec<String>) -> Option<Vec<NetSegment>> {
    let route_info_vec = match parse_routing_table() {
        Ok(x) => x,
        Err(e) => {
            warn!("get_lans {:?}", e);
            return None;
        }
    };
    let net_segment_vec = route_info_vec.into_iter()
        .filter_map(|route_info| {
            if if_name.contains(&route_info.dev) {
                let ip = if route_info.dst == "default" {
                    IpAddr::from(Ipv4Addr::new(0, 0, 0, 0))
                }
                else {
                    IpAddr::from_str(&route_info.dst)
                        .ok()?
                };

                let mask = route_info.mask;

                let gw = IpAddr::from_str(&route_info.gw)
                    .ok();

                Some(NetSegment::new(
                    ip,
                    mask,
                    gw,
                ))
            }
            else {
                None
            }
        })
        .collect::<Vec<NetSegment>>();

    Some(net_segment_vec)
}

#[cfg(test)]
mod test {
    use crate::interface::get_default_interface;

    #[test]
    fn test_get_default_interface() {
        println!("{:?}", get_default_interface());
    }
}