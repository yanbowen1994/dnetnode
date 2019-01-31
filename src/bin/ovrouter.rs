use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::thread::sleep;

#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate clap;
use clap::{App, Arg};

extern crate ovrouter;
use ovrouter::settings::Settings;
use ovrouter::tinc_manager::check::*;
use ovrouter::tinc_manager::Tinc;
use ovrouter::domain::Info;
use ovrouter::http_server_client::Client;
use ovrouter::tinc_manager::install_tinc;


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

    let tinc = Tinc::new(settings.tinc.home_path.clone(), settings.tinc.pub_key_path.clone());

    info!("check_tinc_complete");
    if !check_tinc_complete(&settings.tinc.home_path) {
        info!("install_tinc");
        install_tinc(&settings, &tinc);
    }

    info!("check_pub_key");
    if !check_pub_key(&settings.tinc.home_path, &settings.tinc.pub_key_path) {
        tinc.create_pub_key();
    }

    info!("Get local info.");
    let mut info = Info::new_from_local(&settings);
    info.proxy_info.create_uid();

    let client = Client::new(settings.server.url.clone());
//    login
    {
        if !client.proxy_login(&settings, &mut info) {
            println!("Proxy login failed");
            std::process::exit(1);
        };
    }

//    注册proxy
    info!("proxy_register");

    if !client.proxy_register(&mut info) {
        println!("Proxy register failed");
        std::process::exit(1);
    }

//    heartbeat
    info!("proxy_heart_beat");
    if !client.proxy_heart_beat(&info) {
        println!("Proxy heart beat send failed");
        std::process::exit(1);
    };

    let client_arc = Arc::new(
        Mutex::new(client));

    let tinc_arc = Arc::new(
        Mutex::new(tinc));

    let info_arc = Arc::new(
        Mutex::new(info));

    use std::thread::spawn;
    use ovrouter::http_server_client::web_server;

    let info_arc_clone = info_arc.clone();
    let tinc_arc_clone = tinc_arc.clone();
    let web_handle = spawn(
        move ||web_server(info_arc_clone, tinc_arc_clone));

//    let main_handle = spawn(
//        move ||    main_loop(tinc_arc, client_arc, info_arc, &settings));
    main_loop(tinc_arc, client_arc, info_arc, &settings)
}

fn main_loop(tinc_arc:    Arc<Mutex<Tinc>>,
             client_arc:       Arc<Mutex<Client>>,
             info_arc:         Arc<Mutex<Info>>,
             settings:         &Settings,
) {
    let heartbeat_frequency = Duration::from_secs(20);
//    let landmark_frequency = Duration::from_secs(15600);
    let check_tinc_frequency = Duration::from_secs(3);

    let mut now = Instant::now();
    let mut heartbeat_time = now.clone();
//    let mut landmark_time = now.clone();
    let mut check_tinc_time = now.clone();

    loop {
        if now.duration_since(heartbeat_time) > heartbeat_frequency {
            if let Ok(client) = client_arc.try_lock() {
                if let Ok(info) = info_arc.try_lock() {
                    info!("proxy_heart_beat");
                    if !client.proxy_heart_beat(&info){
                        error!("Heart beat send failed.")
                    };
                    heartbeat_time = now.clone();
                }
            }
        }

        if now.duration_since(check_tinc_time) > check_tinc_frequency {
            let mut lock_or_pass = true;
            info!("check_tinc_status");
            if check_tinc_status(&settings.tinc.home_path) {
                if let Ok(tinc) = tinc_arc.try_lock() {
                    tinc.restart_tinc();
                }
                else {
                    lock_or_pass = false;
                }
            }
            if lock_or_pass {
                check_tinc_time = now.clone();
            }
        }

        sleep(Duration::new(1, 0));
        now = Instant::now();

    }
}