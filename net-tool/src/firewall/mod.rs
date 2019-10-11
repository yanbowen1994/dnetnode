use std::net::IpAddr;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

mod types;

pub use imp::{iptabels_create_chain, iptables_append_rule, iptables_delete_rule, iptables_insert_rule};