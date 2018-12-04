use std::net::{Ipv4Addr, SocketAddr, TcpStream, IpAddr};
use std::io::Result;
use std::time::Duration;

use sys_tool::{cmd};

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