use tinc_plugin::TincOperatorError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Can read pubkey file.")]
    PubkeyFile(#[error(cause)] ::std::io::Error),

    #[error(display = "Can not get tinc dump info.")]
    TincDump(#[error(cause)] ::std::io::Error),

    #[error(display = "Can read tinc-up file.")]
    GetVip(#[error(cause)] TincOperatorError),

    #[error(display = "tinc-up vip setting error.")]
    ParseVip(#[error(cause)] ::std::net::AddrParseError),

    #[error(display = "tinc-up vip setting error.")]
    FileNotExit(#[error(cause)] ::std::io::Error),

    #[error(display = "Get Mac.")]
    GetMac,
}