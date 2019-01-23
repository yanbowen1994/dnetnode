use std::process::exit;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Instant, Duration};
use std::thread::sleep_ms;

#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate clap;
use clap::{App, Arg};

extern crate ovrouter;
use ovrouter::settings::Settings;
use ovrouter::tinc_manager::check::*;
use ovrouter::tinc_manager::Operater;
use ovrouter::net_tool::url_get;
use ovrouter::domain::Info;
use ovrouter::http_server_client::Client;


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

    if !check_tinc_complete(tinc_home) {
        println!("No tinc install in ".to_string() + tinc_home);
        exit(1);
    }

    if !check_pub_key(tinc_home, pub_key_path) {
        Operater::new(&settings.tinc.home_path, &settings.tinc.pub_key_path).create_pub_key();
    }

    let tinc_operater = Arc::new(
        Mutex::new(
            Operater::new(&settings.tinc.home_path, &settings.tinc.pub_key_path)));

    let info = Info::new_from_local(&settings);

    let client = Arc::new(
        Mutex::new(
            Client::new("https://192.168.9.38/".to_string(), &info)));
//    注册proxy
//    client_arc.lock().unwrap()

//    web_server();
    main_loop(tinc_operater, client, &settings);
}

fn main_loop(tinc_operater: Arc<Mutex<Operater>>,
                 client_arc: Arc<Mutex<Client>>,
                 settings: &Settings,
) {
    let heartbeat_frequency = Duration::from_secs(20);
    let landmark_frequency = Duration::from_secs(15600);
    let check_tinc_frequency = Duration::from_secs(3);

    let mut now = Instant::now();
    let mut heartbeat_time = now.clone();
    let mut landmark_time = now.clone();
    let mut check_tinc_time = now.clone();

    loop {
        if now.duration_since(heartbeat_time) > heartbeat_frequency {
            if let Ok(client) = client_arc.try_lock() {
                client.proxy_heart_beat();
                heartbeat_time = now.clone();
            }
        }

        if now.duration_since(landmark_time) > landmark_frequency {
            upload_proxy_status(conductor_url, info);
            landmark_time = now.clone();
        }

        if now.duration_since(check_tinc_time) > check_tinc_frequency {
            let mut lock_or_pass = true;
            if check_tinc_status(&settings.tinc.home_path) {
                if let Ok(operater) = tinc_operater.try_lock() {
                    operater.restart_tinc();
                }
                else {
                    lock_or_pass = false;
                }
            }
            if lock_or_pass {
                check_tinc_time = now.clone();
            }
        }

        sleep_ms(1);
        now = Instant::now();

    }
}