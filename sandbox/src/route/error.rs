pub type Result<T> = std::result::Result<T, Error>;

#[allow(non_camel_case_types)]
#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "exec_route_cmd")]
    exec_route_cmd(#[error(cause)] std::io::Error),

    #[error(display = "exec_ip_route_cmd")]
    exec_ip_route_cmd(#[error(cause)] std::io::Error),

    #[error(display = "parse_route_cmd")]
    parse_route_cmd,

    #[error(display = "parse_ip_route_cmd")]
    parse_ip_route_cmd,

    #[error(display = "default_route_not_found")]
    default_route_not_found,

    #[error(display = "get_mac_address")]
    get_mac_address(#[error(cause)] mac_address::MacAddressError),

    #[error(display = "get_mac_address_empty")]
    get_mac_address_empty,
}
