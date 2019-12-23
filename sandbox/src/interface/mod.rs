use std::net::IpAddr;
use std::str::FromStr;
use crate::route::get_default_route;
use dnet_types::team::NetSegment;

pub fn get_default_interface() -> Option<NetSegment> {
    let default_route = get_default_route()?;
    println!("{:?}", default_route);
    let ip = match IpAddr::from_str(&default_route.dst) {
        Ok(x) => x,
        Err(_) => {
            if default_route.dst == "default" {
                IpAddr::from_str("0.0.0.0").unwrap()
            }
            else {
                return None;
            }
        }
    };
    let gw = IpAddr::from_str(&default_route.gw)
        .ok();

    Some(NetSegment {
        ip,
        mask: default_route.mask,
        gw,
    })
}

#[cfg(test)]
mod test {
    use crate::interface::get_default_interface;

    #[test]
    fn test_get_default_interface() {
        println!("{:?}", get_default_interface());
    }
}