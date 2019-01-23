use std::process::exit;
use std::sync::{Arc, Mutex, RwLock};

#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate clap;
use clap::{App, Arg};

extern crate ovrouter;
use ovrouter::settings::Settings;
use ovrouter::tinc_manager::check::*;
use ovrouter::tinc_manager::install_tinc::create_pub_key;
use ovrouter::tinc_manager::Operater;
use ovrouter::http_server_client::client::upload_proxy_status;

fn main() {
    let matches =  App::new("ovrouter")
            .arg(Arg::with_name("debug")
            .short("d")
            .long("debug")
            .value_name("log_level")
            .takes_value(true))
            .get_matches();

    match matches.value_of("debug") {
        Some(log_level) => {
            match log_level {
                _ if log_level == "0" => simple_logger::init_with_level(log::Level::Error).unwrap(),
                _ if log_level == "1" => simple_logger::init_with_level(log::Level::Warn).unwrap(),
                _ if log_level == "2" => simple_logger::init_with_level(log::Level::Info).unwrap(),
                _ if log_level == "3" => simple_logger::init_with_level(log::Level::Debug).unwrap(),
                _ if log_level == "4" => simple_logger::init_with_level(log::Level::Trace).unwrap(),
                _  => (),
            }
        }
        None => ()
    }

    let settings:Settings = Settings::load_config().expect("Error: can not parse settings.toml");

    let tince_operater = Arc::new(
        Mutex::new(
        Operater::new(&settings.tinc.home_path, &settings.tinc.pub_key_path)));

    if !check_tinc_complete(tinc_home) {
        println!("No tinc install in ".to_string() + tinc_home);
        exit(1);
    }

    if !check_pub_key(tinc_home, pub_key_path) {
        create_pub_key(tinc_home, pub_key_path)
    }
    web_server();
    main_loop();
}
