use std::path::PathBuf;

#[macro_use]
extern crate log;
extern crate clap;
use clap::{App, Arg};

extern crate ovrouter;

use ovrouter::settings::{self, Settings, get_settings};
use ovrouter::daemon::Daemon;
use ovrouter::logging::init_logger;

use std::convert::From;

const LOG_FILENAME: &str = "dnetovr.log";
#[cfg(unix)]
const DEFAULT_LOG_DIR: &str = "/var/log/dnetovr/";
#[cfg(unix)]
const DEFAULT_CONFIG_DIR: &str = "/root/dnetovr";

pub const COMMIT_ID: &str = include_str!(concat!(env!("OUT_DIR"), "/git-commit-id.txt"));

pub const COMMIT_DATE: &str = include_str!(concat!(env!("OUT_DIR"), "/git-commit-date.txt"));

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Can find settings.toml, please use --config to specify configuration file.")]
    NoSettingFile,

    #[error(display = "Can not parse settings.toml.")]
    ParseSetting(#[error(cause)] settings::Error),
    #[error(display = "Can not create log dir.")]
    CreateLogDir(#[error(cause)] ::std::io::Error),
    #[error(display = "Init Daemon failed.")]
    DaemonInit(#[error(cause)] ovrouter::daemon::Error),
}

fn main() {
    let exit_code = match init() {
        Ok(_) => 0,
        Err(error) => {
            error!("{:?}\n{}", error, error);
            1
        }
    };
    debug!("Process exiting with code {}", exit_code);
    std::process::exit(exit_code);
}

fn init() -> Result<()>{
    // 命令行提示
    let matches =  App::new("dnetovr v1.0.5.0")
        .version(&format!("\nCommit date: {}\nCommit id:   {}", COMMIT_DATE, COMMIT_ID).to_string()[..])
        .args(&vec![
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .value_name("log_level")
                .takes_value(true),
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("config_dir")
                .takes_value(true)
        ])
        .get_matches();

    let mut log_level = log::LevelFilter::Off;
    match matches.value_of("debug") {
        Some(arg_log_level) => {
            match arg_log_level {
                "0" => log_level = log::LevelFilter::Error,
                "1" => log_level = log::LevelFilter::Warn,
                "2" => log_level = log::LevelFilter::Info,
                "3" => log_level = log::LevelFilter::Debug,
                "4" => log_level = log::LevelFilter::Trace,
                _ => (),
            }
        }
        None => ()
    }

    let config_dir = match matches.value_of("config") {
        Some(x) => x,
        None => DEFAULT_CONFIG_DIR,
    };
    // 解析settings.toml文件
    Settings::load_config(config_dir)
        .map_err(|e|{
            let err = Error::ParseSetting(e);
            println!("{:?}\n{}", err, err);
            err
        })?;
    let settings = get_settings();

    let settings_log_level = settings.client.log_level.clone();

    match settings_log_level {
        Some(settings_log_level) => {
            match &settings_log_level[..] {
                "Error" => log_level = log::LevelFilter::Error,
                "Warn" => log_level = log::LevelFilter::Warn,
                "Info" => log_level = log::LevelFilter::Info,
                "Debug" => log_level = log::LevelFilter::Debug,
                "Trace" => log_level = log::LevelFilter::Trace,
                _  => (),
            }
        }
        None => ()
    }

    let mut _log_dir: PathBuf = match settings.client.log_dir.clone() {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(DEFAULT_LOG_DIR),
    };

    if !std::path::Path::new(&_log_dir).is_dir() {
        std::fs::create_dir_all(&_log_dir).map_err(Error::CreateLogDir)?;
    }

    let log_file = _log_dir.join(LOG_FILENAME);

    if let Err(e) = init_logger(
        log_level,
        Some(&log_file),
        true,
    ) {
        println!("Error: Can't start logger.\n{:?}", e);
        std::process::exit(1);
    }

    let mut daemon = Daemon::start().map_err(Error::DaemonInit)?;
    daemon.run();
    Ok(())
}