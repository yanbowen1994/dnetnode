extern crate config;
use self::config::{ConfigError, Config, File};
use std::path::Path;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Can find settings.toml, please use --config to specify configuration file.")]
    NoSettingFile,

    #[error(display = "Can not parse settings")]
    ConfigError(ConfigError)
}

#[derive(Clone, Debug, Deserialize)]
pub struct Common {
    pub home_path:       Option<String>,
    pub log_level:       Option<String>,
    pub log_dir:         Option<String>,
    pub mode:            Option<String>,
    pub conductor_url:   Option<String>,
    pub username:        Option<String>,
    pub password:        Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Proxy {
    pub local_port:                             Option<String>,
    pub local_https_server_certificate_file:    Option<String>,
    pub local_https_server_privkey_file:        Option<String>,
    pub proxy_type:                             Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Client {
    pub auto_connect:                              Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct FileSettings {
    pub common: Option<Common>,
    pub proxy:  Option<Proxy>,
    pub client:   Option<Client>,
}

impl FileSettings {
    pub(crate) fn load_config(config_dir: &str) -> Result<FileSettings, Error> {
        let mut settings = Config::new();

        let config_file = config_dir.to_owned() + "/settings.toml";

        if !Path::new(&config_file).is_file() {
            println!("The configuration file could not be found. Please use --config to specify the configuration directory.");
            return Err(Error::NoSettingFile);
        }

        settings
            .merge(File::with_name(&(config_dir.to_owned() + "/settings.toml")))
            .expect("Error: Can not parse settings.");

        let mut settings: FileSettings = settings.try_into().map_err(Error::ConfigError)?;
        Ok(settings)
    }
}


#[test]
fn test_setting() {
    Settings::load_config("/root/Rust/ovrouter_Rust")
        .map_err(|e|{
            eprintln!("{:?}\n{}", e, e);
        })
        .expect("Error: Can not parse settings.");
}