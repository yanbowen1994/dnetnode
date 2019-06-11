use std::path::PathBuf;

#[macro_use]
extern crate log;
extern crate clap;
use clap::{App, Arg};

extern crate ovrouter;

use ovrouter::settings::{self, Settings};
use ovrouter::daemon::Daemon;
use ovrouter::tinc_manager::TincOperator;
use ovrouter::domain::Info;
use ovrouter::http_server_client::Client;
use ovrouter::http_server_client::web_server;
use ovrouter::logging::init_logger;
use ovrouter::sys_tool::datetime;

use std::convert::From;

const LOG_FILENAME: &str = "ovrouter.log";
#[cfg(unix)]
const DEFAULT_LOG_DIR: &str = "/var/log/ovr/";

pub const COMMIT_DATE: &str = include_str!(concat!(env!("OUT_DIR"), "/git-commit-date.txt"));

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
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
            error!("{:?} {}", error, error);
            1
        }
    };
    debug!("Process exiting with code {}", exit_code);
    std::process::exit(exit_code);
}

fn init() -> Result<()>{
//    let date = &datetime()[..];
    // 命令行提示
    let matches =  App::new("ovrouter")
        .version( COMMIT_DATE)
        .arg(Arg::with_name("debug")
            .short("d")
            .long("debug")
            .value_name("log_level")
            .takes_value(true))
        .get_matches();

    let mut arg_log_level = log::LevelFilter::Off;
    match matches.value_of("debug") {
        Some(log_level) => {
            match log_level {
                _ if log_level == "0" => arg_log_level = log::LevelFilter::Error,
                _ if log_level == "1" => arg_log_level = log::LevelFilter::Warn,
                _ if log_level == "2" => arg_log_level = log::LevelFilter::Info,
                _ if log_level == "3" => arg_log_level = log::LevelFilter::Debug,
                _ if log_level == "4" => arg_log_level = log::LevelFilter::Trace,
                _ => (),
            }
        }
        None => ()
    }

    // 解析settings.toml文件
    let settings:Settings = Settings::load_config().map_err(Error::ParseSetting)?;
    let log_level = settings.client.log_level.clone();

    let mut setting_log_level = log::LevelFilter::Off;
    match log_level {
        Some(log_level) => {
            match log_level {
                _ if log_level == "Error" => setting_log_level = log::LevelFilter::Error,
                _ if log_level == "Warn" => setting_log_level = log::LevelFilter::Warn,
                _ if log_level == "Info" => setting_log_level = log::LevelFilter::Info,
                _ if log_level == "Debug" => setting_log_level = log::LevelFilter::Debug,
                _ if log_level == "Trace" => setting_log_level = log::LevelFilter::Trace,
                _  => (),
            }
        }
        None => ()
    }

    let mut log_level = log::LevelFilter::Off;
    if arg_log_level != log_level {
        log_level = arg_log_level;
    }
    else if arg_log_level != log_level {
        log_level = setting_log_level;
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

    let mut daemon = Daemon::start(settings).map_err(Error::DaemonInit)?;
    daemon.run();
    Ok(())
}