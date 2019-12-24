use crate::route::error::Error as RouteError;

pub type Result<T> = std::result::Result<T, Error>;

#[allow(non_camel_case_types)]
#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "exec_route_cmd")]
    route_table(#[error(cause)] RouteError),

    #[error(display = "default_interface_not_found")]
    default_interface_not_found,
}
