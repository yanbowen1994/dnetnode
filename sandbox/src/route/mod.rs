use std::net::IpAddr;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;
pub mod types;

pub use imp::{
    add_route,
    del_route,
    is_in_routing_table,
    parse_routing_table,
    parse_netmask_to_cidr,
};