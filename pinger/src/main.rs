extern crate socket2_test;

use std::time::Duration;
use std::net::IpAddr;

fn main() {
    let addr = IpAddr::from([8, 8, 8, 8]);
    let timeout = Some(Duration::from_secs(7));
    let ttl = None;
    let ident = None;
    let seq_cnt = Some(3);
    let payload = None;
    let res = socket2_test::ping(addr, timeout, ttl, ident, seq_cnt, payload);
    println!("{:?}", res);
}