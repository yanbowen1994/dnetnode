use config::ConfigError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
#[allow(non_camel_case_types)]
pub enum Error {
    #[error(display = "Get router SN")]
    GetSN,

    #[error(display = "Can find settings.toml, please use --config to specify configuration file.")]
    NoSettingFile,

    #[error(display = "Can not parse settings")]
    ConfigError(ConfigError),

    #[error(display = "Process Home Path Not Set.")]
    home_path_not_set,

    #[error(display = "conductor_url_not_set")]
    conductor_url_not_set,

    #[error(display = "invalid_conductro_url")]
    invalid_conductro_url,

    #[error(display = "https_server_privkey_not_found")]
    https_server_privkey_not_found,

    #[error(display = "https_server_certificate_not_found")]
    https_server_certificate_not_found,

    #[error(display = "Using for trance Option to Result")]
    NoneError,
}