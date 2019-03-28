extern crate config;
use self::config::{ConfigError, Config, File};

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
    pub server_port: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub client: Client,
    pub tinc: Tinc,
}

impl Settings {
    pub fn load_config() -> Result<Self, ConfigError> {
        let mut settings = Config::new();

        settings
            .merge(File::with_name("settings.toml"))
            .expect("Error: Can not parse settings.");

        settings.try_into()

    }
}

#[test]
fn test_setting() {
    let _a = Settings::load_config().expect("Error: Can not parse settings.");
}