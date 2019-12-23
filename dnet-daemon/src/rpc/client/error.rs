pub type Result<T> = std::result::Result<T, Error>;

#[allow(non_camel_case_types)]
#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Connection with conductor timeout")]
    RpcTimeout,

    #[error(display = "Connection with conductor timeout")]
    TeamNotFound,

    #[error(display = "Monitor init failed.")]
    InitMonitor,
}