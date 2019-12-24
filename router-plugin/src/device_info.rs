#[cfg(not(target_arch = "arm"))]
use std::net::IpAddr;
#[cfg(target_arch = "arm")]
use std::net::Ipv4Addr;
use std::str::FromStr;

#[cfg(target_arch = "arm")]
extern crate nix;
#[cfg(target_arch = "arm")]
use nix::sys::socket::{AddressFamily, SockAddr};
#[cfg(target_arch = "arm")]
use sandbox::interface::get_lans;
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
                // for test!
                let lan1 = NetSegment {
                    ip:     IpAddr::from_str("192.168.113.1").unwrap(),
                    mask:   24,
                    gw:     Some(IpAddr::from_str("192.168.113.1").unwrap()),
                };
                let lan2 = NetSegment {
                    ip:     IpAddr::from_str("192.168.114.1").unwrap(),
                    mask:   24,
                    gw:     Some(IpAddr::from_str("192.168.114.1").unwrap()),
                };
                let lan = vec![lan1, lan2];
                return Some(DeviceInfo {
                    lan,
                    cloud_led_on: false,
                });
            }
        #[cfg(target_arch = "arm")]
            {
                let lan = get_lans(vec![
                    "br0".to_owned(),
                    "br1".to_owned(),
                    "br2".to_owned(),
                    "br3".to_owned(),
                ])?;
                if lan.len() == 0 {
                    return None;
                }
                return Some(DeviceInfo {
                    lan,
                    cloud_led_on: false,
                });
            }
    }
}
