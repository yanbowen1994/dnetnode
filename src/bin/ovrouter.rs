use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::thread::{sleep, spawn};

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
use ovrouter::http_server_client::web_server;


fn main() {
    // 命令行提示
    let matches =  App::new("ovrouter")
        .arg(Arg::with_name("debug")
            .short("d")
            .long("debug")
            .value_name("log_level")
            .takes_value(true))
        .get_matches();

    // 设置debug等级
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

    // 解析Settings.toml文件
    let settings:Settings = Settings::load_config().expect("Error: can not parse settings.toml");

    // 初始化tinc操作
    let tinc = Tinc::new(settings.tinc.home_path.clone(), settings.tinc.pub_key_path.clone());

    // 监测tinc文件完整性，失败将安装tinc
    info!("check_tinc_complete");
    if !check_tinc_complete(&settings.tinc.home_path) {
        info!("install_tinc");
        install_tinc(&settings, &tinc);
    }

    // 监测tinc pub key 不存在或生成时间超过一个月，将生成tinc pub key
    info!("check_pub_key");
    if !check_pub_key(&settings.tinc.home_path, &settings.tinc.pub_key_path) {
        tinc.create_pub_key();
    }

    // 获取本地 tinc geo 和 ip信息，创建proxy uuid
    info!("Get local info.");
    let mut info = Info::new_from_local(&settings);
    info.proxy_info.create_uid();

    // 初始化上报操作
    let client = Client::new(settings.server.url.clone());

    // Client Login
    {
        if !client.proxy_login(&settings, &mut info) {
            println!("Proxy login failed");
            std::process::exit(1);
        };
    }

    // 注册proxy
    info!("proxy_register");
    if !client.proxy_register(&mut info) {
        println!("Proxy register failed");
        std::process::exit(1);
    }

    // 初次上传heartbeat
    info!("proxy_heart_beat");
    if !client.proxy_heart_beat(&info) {
        println!("Proxy heart beat send failed");
        std::process::exit(1);
    };

    // 添加多线程 同步操作锁
    // 目前client，仅用于main loop，上传心跳
    let client_arc = Arc::new(
        Mutex::new(client));

    // tinc操作 main loop：监测tinc运行，修改pub key
    // web_server：添加hosts
    let tinc_arc = Arc::new(
        Mutex::new(tinc));

    // 信息包括 geo信息：初次启动获取，目前初始化后无更新
    //          tinc信息： 本机tinc运行参数
    //          proxy信息：公网ip， uuid等
    //          目前 初始化后 main loop 和web_server 都只做读取
    let info_arc = Arc::new(
        Mutex::new(info));

    let info_arc_clone = info_arc.clone();
    let tinc_arc_clone = tinc_arc.clone();

    // 启动web_server,线程
    spawn(move ||web_server(info_arc_clone, tinc_arc_clone));

    // 进入主循环
    main_loop(tinc_arc, client_arc, info_arc, &settings)
}

fn main_loop(tinc_arc:    Arc<Mutex<Tinc>>,
             client_arc:       Arc<Mutex<Client>>,
             info_arc:         Arc<Mutex<Info>>,
             settings:         &Settings,
) {
    // 设置心跳和监测tinc状态的频率，单位：秒
    let heartbeat_frequency = Duration::from_secs(20);
    let check_tinc_frequency = Duration::from_secs(3);

    // now: 当前时间
    // heartbeat_time：前次心跳发送时间
    // check_tinc_time： 前次监测tinc时间
    let mut now = Instant::now();
    let mut heartbeat_time = now.clone();
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

        // 如果监测tinc运行 失败
        //     尝试获取tinc操作
        //         锁获取成功 重启tinc, 重启失败报错 不退出程序
        //         锁获取失败 不更新tinc监测时间等待 1秒后重试
        if now.duration_since(check_tinc_time) > check_tinc_frequency {
            let mut lock_or_pass = true;
            info!("check_tinc_status");
            if !check_tinc_status(&settings.tinc.home_path) {
                if let Ok(tinc) = tinc_arc.try_lock() {
                    tinc.restart_tinc();
                    if !check_tinc_status(&settings.tinc.home_path) {
                        error!("Restart tinc failed");
                    }
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