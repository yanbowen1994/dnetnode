#[cfg(target_arch = "arm")]
use std::net::Ipv4Addr;
#[cfg(target_arch = "arm")]
use std::str::FromStr;

#[cfg(target_arch = "arm")]
extern crate nix;
#[cfg(target_arch = "arm")]
use nix::sys::socket::{AddressFamily, SockAddr};
use dnet_types::team::NetSegment;

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub lan:            Vec<NetSegment>,
    pub cloud_led_on:   bool,
}

impl DeviceInfo {
    pub fn get_info() -> Option<Self> {
        #[cfg(not(target_arch = "arm"))]
            {
                return Some(DeviceInfo {
                    lan: vec![],
                    cloud_led_on: false,
                });
            }
        #[cfg(target_arch = "arm")]
            {
                let mut lan = Self::get_lans(vec![
                    "br0".to_owned(),
                    "br1".to_owned(),
                    "br2".to_owned(),
                    "br3".to_owned(),
                ]);
                if lan.len() == 0 {
                    return None;
                }
                return Some(DeviceInfo {
                    lan,
                    cloud_led_on: false,
                });
            }
    }

    #[cfg(target_arch = "arm")]
    fn get_lans(if_name: Vec<String>) -> Vec<NetSegment> {
        let mut net_segment = vec![];
        for interface in nix::ifaddrs::getifaddrs().unwrap() {
            let mut lan_ip = None;
            let mut mask = None;
            if if_name.contains(&interface.interface_name) && interface.address.unwrap().family() == AddressFamily::Inet {
                if let Some(sock) = interface.address {
                    if let SockAddr::Inet(sock) = sock {
                        lan_ip = Some(sock.to_std().ip());
                    }
                };

                if let Some(sock) = interface.netmask {
                    if let SockAddr::Inet(sock) = sock {
                        mask = Some(sock.to_std().ip());
                    }
                };
            }
            if lan_ip.is_some() && mask.is_some() {
                if let Some(mask) = parse_netmask_to_cidr(&mask.unwrap().to_string()) {
                    net_segment.push(NetSegment {
                        ip: lan_ip.unwrap(),
                        mask,
                    });
                }
            }
        }
        net_segment
    }
}

// CIDR classless inter-domain routing
#[cfg(target_arch = "arm")]
fn parse_netmask_to_cidr(netmask: &str) -> Option<u32> {
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
