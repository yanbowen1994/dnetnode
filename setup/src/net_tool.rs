#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unknown_lints)]

use std::str::FromStr;
use std::net::{Ipv4Addr, SocketAddr, TcpStream, IpAddr};

use std::time::Duration;

use crate::sys_tool::{cmd};

use std::io::{stdout, Write, Read};

pub fn get_wan_name() -> Option<String> {
    let local_ip = get_local_ip().unwrap().to_string();

    let (code, output) = cmd(
        "ip address|grep ".to_string() + &local_ip + " | awk '{print $(7)}'");

    if code != 0 {
        return None;
    }

    Some(output)
}

// 连接8.8.8.8 或8.8.4.4 获取信号输出网卡ip，多网卡取路由表默认外网连接ip
// get_localip().unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))
pub fn get_local_ip() -> std::io::Result<IpAddr> {
    let timeout = Duration::new(3 as u64, 0 as u32);
    let addr0 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8,8,8,8)), 53);
    // 如果可以连接到8.8.8.8 || 8.8.4.4 获取出口ip，如果失败获取网卡ip
    let socket = match TcpStream::connect_timeout(&addr0, timeout) {
        Ok(x) => x,
        Err(_) => {
            let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8,8,4,4)), 53);
            let socket1 = match TcpStream::connect_timeout(&addr1, timeout) {
                Ok(x) => x,
                Err(e) => {
                    let (code, output) = cmd(
                        "ip address|grep -w inet | awk 'NR == 2' | awk '{print $(2)}'".to_string());
                    if !code == 0 {
                        return Err(e);
                    }
                    let ip: Vec<&str> = output.split("/").collect();
                    match IpAddr::from_str(ip[0]){
                        Ok(ip) => return Ok(ip),
                        Err(_) => return Err(e),
                    };
                }
            };
            socket1
        }
    };
    let ip = socket.local_addr()?.ip();
    Ok(ip)
}

pub fn get_mac() -> Option<String> {
    let if_name = get_wan_name().unwrap();
    let (code, output) = cmd(
        "ifconfig|grep ".to_string() + &if_name + "| awk '{print $5}'");

    if code != 0 {
        return None;
    }
    Some(output)

}