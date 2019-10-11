pub type Result<T> = std::result::Result<T, Error>;

#[allow(non_camel_case_types)]
#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "mqtt_connect_error.")]
    mqtt_connect_error,

    #[error(display = "mqtt_client_error.")]
    mqtt_client_error,

    #[error(display = "mqtt_msg_parse_failed.")]
    mqtt_msg_parse_failed(String),
}