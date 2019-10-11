use std::net::IpAddr;
use std::str::FromStr;
use std::process::Command;

// netmask CIDR
pub fn add_route(ip: &IpAddr, netmask: &str, dev: &str) {
    #[cfg(not(target_os = "windows"))]
        {
            let ip_mask = ip.clone().to_string() + "/" + netmask;
            let _ = Command::new("ip").args(vec!["route", "add", &ip_mask, "dev", dev]).spawn();
        }
}

pub fn del_route(ip: &IpAddr, netmask: &str, dev: &str) {
    #[cfg(not(target_os = "windows"))]
        {
            let ip_mask = ip.clone().to_string() + "/" + netmask;
            let _ = Command::new("ip").args(vec!["route", "del", &ip_mask, "dev", dev]).spawn();
        }
}

#[test]
fn test() {
    let ip = IpAddr::from_str("12.12.12.12").unwrap();
    add_route(&ip, "32", "enp3s0");

    let stdout = duct::cmd!("route").stdout_capture().run().unwrap().stdout;
    let stdout = String::from_utf8(stdout).unwrap();
    assert!(stdout.contains("12.12.12.12"));
    del_route(&ip, "32", "enp3s0");

    let stdout = duct::cmd!("route").stdout_capture().run().unwrap().stdout;
    let stdout = String::from_utf8(stdout).unwrap();
    assert!(!stdout.contains("12.12.12.12"));
}