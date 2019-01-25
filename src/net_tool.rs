#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unknown_lints)]

use std::net::{Ipv4Addr, SocketAddr, TcpStream, IpAddr};
use std::io::Result;
use std::time::Duration;

use rustc_serialize::json::{decode, encode};
//use rustc_serialize::json;
use serde_json::to_vec;

use json;
extern crate openssl;
use self::openssl::ssl;

use sys_tool::{cmd};

//for url get
extern crate curl;

use std::io::{stdout, Write, Read};

use self::curl::easy::Easy;


pub fn get_wan_name() -> Option<String> {
    let local_ip = get_local_ip().unwrap().to_string();

    let (code, output) = super::sys_tool::cmd(
        "ip address|grep ".to_string() + &local_ip + " | awk '{print $(7)}'");

    if code != 0 {
        return None;
    }

    Some(output)
}



// 连接8.8.8.8:53 获取信号输出网卡ip，多网卡取路由表默认外网连接ip
// get_localip().unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))
pub fn get_local_ip() -> Result<IpAddr> {
    let timeout = Duration::new(10 as u64, 0 as u32);
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8,8,8,8)), 53);

    let socket = try!(TcpStream::connect_timeout(&socket, timeout));
    let ip = try!(socket.local_addr()).ip();
    Ok(ip)
}


pub fn url_get(url:&str) -> Option<(String, u32)> {
    let mut res_data = Vec::new();

    let mut handle = Easy::new();

    handle.timeout(Duration::new(5, 0));
    handle.show_header(false).unwrap();
    handle.url(url).unwrap();
    handle.perform().unwrap_or(return None);

    {
        let mut tran = handle.transfer();
        let _ = tran.write_function(|buf| {
            res_data.extend_from_slice(buf);
            Ok(buf.len())
        });
        tran.perform().unwrap_or(return None);
    }

    let res_data = String::from_utf8_lossy(&res_data).into_owned();
    return Some((res_data, handle.response_code().unwrap_or(return None)));
}

pub fn url_post(url: &str, data: String) -> Option<(String, u32)> {
    let data = http_json(data);

    let mut send_data = data.as_bytes();

    let mut res_data = Vec::new();

    let mut handle = Easy::new();

    handle.timeout(Duration::new(10, 0));
    handle.show_header(false).unwrap();
    handle.post(true).unwrap();
    handle.post_field_size(send_data.len() as u64).unwrap();
    handle.url(url).unwrap();
    handle.ssl_verify_host(false).unwrap();
    handle.ssl_verify_peer(false).unwrap();
//    use file_tool::File;
//    let fd = File::new("/root/key.pem".to_string());
//    handle.ssl_cert(fd.read());

    {
        let mut tracsfer = handle.transfer();
        tracsfer.read_function(move |into| {
            Ok(send_data.read(into).unwrap_or(0))
        }).unwrap();

        let _ = tracsfer.write_function(|buf| {
            res_data.extend_from_slice(buf);
            Ok(buf.len())
        });

        match tracsfer.perform() {
            Ok(_) => (),
            Err(E) => {
                println!("{:?}", E);
                return None;
            }
        };
    }

    let res = String::from_utf8_lossy(&res_data).into_owned();

    let mag = res;
    let code = handle.response_code().unwrap();
    return Some((mag, code));
}

pub fn http_json(json_str: String) -> String {
    let mut json_str = json_str.clone();
    json_str.replace("\"", "")
        .replace(":", "=")
        .replace(",", "&")
        .replace("{", "")
        .replace("}", "")
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