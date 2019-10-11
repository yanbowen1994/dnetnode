use std::net::IpAddr;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

pub use imp::{add_route, del_route};