#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(windows)]
#[path = "windows.rs"]
mod imp;

mod batch_route;
pub use batch_route::batch_route;
pub mod types;

#[cfg(windows)]
pub use imp::{
    add_route,
    del_route,
    is_in_routing_table,
    parse_netmask_to_cidr,
};

#[cfg(unix)]
pub use imp::{
    add_route,
    del_route,
    is_in_routing_table,
    parse_routing_table,
    parse_netmask_to_cidr,
};