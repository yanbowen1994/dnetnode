use std::net::IpAddr;

use super::*;

pub fn batch_route(adds: &Vec<IpAddr>, dels: &Vec<IpAddr>, dev: &str) {
    #[cfg(unix)]
        {
            let now_route = parse_routing_table();
            for add in adds {
                if !is_in_routing_table(
                    &now_route,
                    add,
                    32,
                    dev) {
                    add_route(add, 32, dev)
                }
            }

            for del in dels {
                if is_in_routing_table(
                    &now_route,
                    del,
                    32,
                    dev) {
                    del_route(del, 32, dev)
                }
            }
        }
    #[cfg(windows)]
        {
            for add in adds {
                add_route(add, 32, TINC_INTERFACE)
            }

            for del in dels {
                del_route(del, 32, TINC_INTERFACE)
            }
        }
}
