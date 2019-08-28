extern crate config;
use self::config::{ConfigError, Config, File};
use std::path::Path;

static mut EL: *mut Settings = 0 as *mut _;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Can find settings.toml, please use --config to specify configuration file.")]
    NoSettingFile,

    #[error(display = "Can not parse settings")]
    ConfigError(ConfigError)
}

#[derive(Clone, Debug, Deserialize)]
pub struct Tinc {
    pub home_path: String,
    pub pub_key_path: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Server {
    pub url: String,
    pub geo_url: String,

}
#[derive(Clone, Debug, Deserialize)]
pub struct Client {
    pub log_level: Option<String>,
    pub log_dir: Option<String>,
    pub username: String,
    pub password: String,
    pub local_port: String,
    pub local_https_server_certificate_file: String,
    pub local_https_server_privkey_file: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub client: Client,
    pub tinc: Tinc,
}

impl Settings {
    pub fn load_config(config_dir: &str) -> Result<(), Error> {
        let mut settings = Config::new();

        let config_file = config_dir.to_owned() + "/settings.toml";

        if !Path::new(&config_file).is_file() {
            println!("The configuration file could not be found. Please use --config to specify the configuration directory.");
            return Err(Error::NoSettingFile);
        }

        settings
            .merge(File::with_name(&(config_dir.to_owned() + "/settings.toml")))
            .expect("Error: Can not parse settings.");

        let mut settings: Settings = settings.try_into().map_err(Error::ConfigError)?;

        let now = chrono::Utc::now().to_string();
        settings.last_runtime = Some(now);

        unsafe {
            EL = Box::into_raw(Box::new(settings));
        };
        Ok(())
    }
}

pub fn get_settings() -> &'static Settings {
    unsafe {
        if EL == 0 as *mut _ {
            panic!("Get settings instance, before init");
        }
        &mut *EL
    }
}

pub fn get_settings_mut() -> &'static mut Settings {
    unsafe {
        if EL == 0 as *mut _ {
            panic!("Get settings instance, before init");
        }
        &mut *EL
    }
}

#[test]
fn test_setting() {
    Settings::load_config("/root/dnetovr")
        .map_err(|e|{
            eprintln!("{:?}\n{}", e, e);
        })
        .expect("Error: Can not parse settings.");
    let _settings = get_settings();
}