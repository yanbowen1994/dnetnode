use super::client;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "client::Error")]
    Client(#[error(cause)] client::Error),
}