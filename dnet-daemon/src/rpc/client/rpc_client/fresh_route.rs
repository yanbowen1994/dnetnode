use std::net::IpAddr;

use sandbox::route;

use crate::settings::default_settings::TINC_INTERFACE;

pub fn fresh_route(adds: &Vec<IpAddr>, dels: &Vec<IpAddr>) {
    #[cfg(unix)]
        {
            let now_route = route::parse_routing_table();
            for add in adds {
                if !route::is_in_routing_table(
                    &now_route,
                    add,
                    32,
                    TINC_INTERFACE) {
                    route::add_route(add, 32, TINC_INTERFACE)
                }
            }

            for del in dels {
                if route::is_in_routing_table(
                    &now_route,
                    del,
                    32,
                    TINC_INTERFACE) {
                    route::del_route(del, 32, TINC_INTERFACE)
                }
            }
        }
    #[cfg(windows)]
        {
            for add in adds {
                route::add_route(add, 32, TINC_INTERFACE)
            }

            for del in dels {
                route::del_route(del, 32, TINC_INTERFACE)
            }
        }
}
