use super::parse_file::FileSettings;

static mut EL: *mut Settings = 0 as *mut _;


#[derive(Clone, Debug, Deserialize)]
pub struct Tinc {
    pub home_path: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Server {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Proxy {
    pub log_level: String,
    pub log_dir: String,
    pub username: String,
    pub password: String,
    pub local_port: String,
    pub local_https_server_certificate_file: String,
    pub local_https_server_privkey_file: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub proxy: Proxy,
    pub tinc: Tinc,
    pub last_runtime: String
}

impl Settings {
    pub fn new(config_dir: &str) {
        let mut settings = FileSettings::load_config(config_dir)
            .map_err(|e|warn!("Can't parse settings.toml. ".to_owned() + &e.to_string()))
            .and_then(|file_seting|
                Self::parse_file_settings(file_seting)
            )
            .unwrap_or_else(|_|Settings::defualt());

        let now = chrono::Utc::now().to_string();
        settings.last_runtime = now;

        unsafe {
            EL = Box::into_raw(Box::new(settings));
        };
        Ok(())
    }

    fn defualt() -> Self {

    }

    fn parse_file_settings(file_settings: FileSettings) -> Self {

    }
}

pub fn get_settings() -> &'static Settings {
    unsafe {
        if EL == 0 as *mut _ {
            panic!("Get settings instance, before init");
        }
        & *EL
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