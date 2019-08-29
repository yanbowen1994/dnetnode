#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unknown_lints)]

use std::str::FromStr;
use std::net::{Ipv4Addr, SocketAddr, TcpStream, IpAddr};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::to_vec;

use sys_tool::{cmd};

//for url get
use reqwest;

pub type Result<T> = std::result::Result<T, reqwest::Error>;

use std::io::{stdout, Write, Read};

pub struct Device {
    pub if_name:    String,
    pub ip:         IpAddr,
    pub mac:        String,
}
impl Device {
    pub fn new() -> Self {
        Device {
            if_name:    "".to_string(),
            ip:         IpAddr::from_str("0.0.0.0").unwrap(),
            mac:        "".to_string(),
        }
    }

    pub fn local() -> Self {
        let mut device = Device::new();
        device.ip = get_local_ip().unwrap();
        device.get_if_name();
        device.get_mac();
        device
    }

    fn get_if_name(&mut self) -> bool {
        let local_ip = self.ip.to_string();
        let (code, output) = cmd(
            "ip address|grep ".to_string() + &local_ip + " | awk '{print $(7)}'");

        if code != 0 {
            return false;
        }
        self.if_name = output;
        return true;
    }

    fn get_mac(&mut self) -> bool {
        let if_name = self.if_name.to_string();
        let (code, output) = cmd(
            "ifconfig|grep ".to_string() + &if_name + "| awk '{print $5}'");

        if code != 0 {
            return false;
        }
        self.mac = output;
        return true;
    }
}

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

/// https post请求
pub fn url_post(url: &str, data: &str, cookie: &str)
    -> Result<reqwest::Response> {
    let res = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(5))
//        .danger_accept_invalid_certs(true)
        .http1_title_case_headers()
        .gzip(false)
        .build()?
        .request(reqwest::Method::POST,
                 reqwest::Url::from_str(url).unwrap())
        .header(reqwest::header::CONTENT_TYPE,
                " application/json;charset=UTF-8")
        .header(reqwest::header::COOKIE,
                cookie)
        .header(reqwest::header::USER_AGENT, "")
        .body(data.to_string())
        .send()?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_connection() {
        assert_eq!(TcpStream::connect("8.8.8.8:53").is_ok(), true);
    }

    #[test]
    fn it_works() {
        //assert_eq!(LOCAL_IP.lock().unwrap().is_none(), true);
        // this will sometimes fail, as I cannot figure out how to control the test order
        let ip1 = get_local_ip().unwrap();
        print!("{}", ip1);
    }
}

/// 将json 解析成 a=1&b=2 格式
pub fn http_json(json_str: String) -> String {
    let json_str = json_str.clone();
    json_str.replace("\\\"", "")
        .replace("\"", "")
        .replace(":", "=")
        .replace(",", "&")
        .replace("{", "")
        .replace("}", "")
}