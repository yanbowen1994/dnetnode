extern crate config;
use self::config::{ConfigError, Config, File};
#[macro_use]

#[derive(Clone, Debug, Deserialize)]
pub struct Tinc {
    pub home_path: String,
    pub pub_key_path: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Geo {
    pub url: String,

}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub geo: Geo,
    pub tinc: Tinc,
}

impl Settings {
    pub fn load_config() -> Result<Self, ConfigError> {
        let mut settings = Config::new();

        settings
            .merge(File::with_name("Settings.toml"))
            .expect("Error: Can not parse settings.");

        settings.try_into()

    }
}

#[test]
fn test_setting() {
    let a = Settings::load_config().expect("Error: Can not parse settings.");
}