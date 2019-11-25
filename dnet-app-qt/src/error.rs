use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[allow(non_camel_case_types)]
pub enum Error {
    #[error(display = "Failed to connect to daemon")]
    DaemonNotRunning(#[error(cause)] io::Error),
    #[error(display = "Failed to connect to daemon")]
    ipc_connect_failed(#[error(cause)] management_client::Error),
}