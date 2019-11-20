use dnet_types::response::Response;

pub type Result<T> = std::result::Result<T, Error>;

#[allow(non_camel_case_types)]
#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Http error.")]
    http(i32),

    #[error(display = "Reqwest Error.")]
    Reqwest(#[error(cause)] reqwest::Error),

    #[error(display = "Parse Ip failed.")]
    UrlParseError,

    #[error(display = "Parse response failed.")]
    ResponseParse(String),
}

impl Error {
    pub fn get_http_error_msg(&self) -> String {
        match &self {
            Error::http(code) => Response::get_msg_by_code(*code),
            _ => String::new(),

        }
    }
}