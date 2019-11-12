use std::path::PathBuf;

#[macro_use]
extern crate log;
use clap::{App, Arg, ArgMatches};

extern crate dnet_daemon;
use dnet_daemon::settings::{Settings, get_settings, Error as SettingsError};
use dnet_daemon::init_logger;

pub const COMMIT_ID: &str = include_str!(concat!(env!("OUT_DIR"), "/git-commit-id.txt"));

pub const COMMIT_DATE: &str = include_str!(concat!(env!("OUT_DIR"), "/git-commit-date.txt"));

pub const LOG_FILENAME: &str = "dnet.log";

pub const DEFAULT_CONFIG_DIR: &str = "/opt/dnet";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Can find settings.toml, please use --config to specify configuration file.")]
    NoSettingFile,
    #[error(display = "Can not parse settings.toml.")]
    ParseSetting(#[error(cause)] SettingsError),
    #[error(display = "Can not create log dir.")]
    CreateLogDir(#[error(cause)] ::std::io::Error),
    #[error(display = "Daemon Error.")]
    DaemonError(#[error(cause)] dnet_daemon::daemon::Error)
}

use dnet_daemon::daemon::Daemon;

fn main() {
    let mut exit_code = match init() {
        Ok(_) => {
            0
        },
        Err(error) => {
            println!("{:?}\n{}", error, error);
            1
        }
    };

    if exit_code == 0 {
        exit_code = match start_daemon() {
            Ok(_) => {
                0
            },
            Err(error) => {
                println!("{:?}\n{}", error, error);
                1
            }
        }
    }

    debug!("Process exiting with code {}", exit_code);
    std::process::exit(exit_code);
}

pub fn init() -> Result<()> {
    // 命令行提示
    let matches =  App::new("dnet 1.0.0")
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

    get_config(&matches)?;

    set_log(&matches)?;

    Ok(())
}

fn get_config(matches: &ArgMatches) -> Result<()> {
    let config_dir = match matches.value_of("config") {
        Some(x) => x,
        None => DEFAULT_CONFIG_DIR,
    };
    // 解析settings.toml文件
    Settings::new(config_dir)
        .map_err(|e|{
            let err = Error::ParseSetting(e);
            println!("{:?}\n{}", err, err);
            err
        })?;
    return Ok(());
}

fn set_log(matches: &ArgMatches) -> Result<()> {
    let settings = get_settings();

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
        None => {
            let settings_log_level = settings.common.log_level.clone();
            match &settings_log_level[..] {
                "Error" => log_level = log::LevelFilter::Error,
                "Warn" => log_level = log::LevelFilter::Warn,
                "Info" => log_level = log::LevelFilter::Info,
                "Debug" => log_level = log::LevelFilter::Debug,
                "Trace" => log_level = log::LevelFilter::Trace,
                _  => (),
            }
        }
    }

    #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
        {
            let mut _log_dir: PathBuf = settings.common.log_dir.clone();

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
        }
    #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
        {
            if let Err(e) = init_logger(
                log_level,
                None,
                true,
            ) {
                println!("Error: Can't start logger.\n{:?}", e);
                std::process::exit(1);
            }
        }

    Ok(())
}

fn start_daemon() -> Result<()> {
    let mut daemon = Daemon::start()
        .map_err(Error::DaemonError)?;
    daemon.run();
    Ok(())
}