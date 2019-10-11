use std::net::IpAddr;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;
pub mod types;

pub use imp::{add_route, del_route, parse_routing_table};