use std::net::IpAddr;

extern crate nix;
use nix::sys::socket::{AddressFamily, SockAddr};

pub struct NetSegment {
    pub ip:     IpAddr,
    pub mask:   IpAddr,
}

pub struct DeviceInfo {
    pub lan:            Vec<NetSegment>,
    pub cloud_led_on:   bool,
}

impl DeviceInfo {
    pub fn get_info() -> Option<Self> {
        let mut lan = get_lans(vec![
            "br0".to_owned(),
            "br1".to_owned(),
            "br2".to_owned(),
            "br3".to_owned(),
        ]);
        if lan.len() == 0 {
            return None;
        }
        Some(DeviceInfo {
            lan,
            cloud_led_on: false,
        })
    }

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
                        mask = sock.to_std().ip();
                    }
                };
            }
            if lan_ip.is_some() && mask.is_some() {
                net_segment.push(NetSegment {
                    ip: lan_ip.unwrap(),
                    mask: mask.unwrap(),
                });
            }
        }
        net_segment
    }
}