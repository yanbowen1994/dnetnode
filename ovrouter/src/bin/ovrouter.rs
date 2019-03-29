use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::thread;
use std::error::Error;
use std::sync::mpsc;
use std::thread::sleep;
use std::path::PathBuf;
//use core::borrow::Borrow;

#[macro_use]
extern crate log;
extern crate clap;
use clap::{App, Arg};

extern crate ovrouter;

use ovrouter::settings::Settings;
use ovrouter::tinc_manager::check::*;
use ovrouter::tinc_manager::Tinc;
use ovrouter::domain::Info;
use ovrouter::http_server_client::Client;
use ovrouter::http_server_client::web_server;
use ovrouter::logging::init_logger;

const LOG_FILENAME: &str = "ovrouter.log";
const DEFAULT_LOG_DIR: &str = "/var/log/ovr/";


fn main() {
    // 命令行提示
    let matches =  App::new("ovrouter 0.1")
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
                _  => (),
            }
        }
        None => ()
    }

    // 解析settings.toml文件
    let settings:Settings = Settings::load_config().expect("Error: can not parse settings.toml");
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
        std::fs::create_dir_all(&_log_dir);
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

    // 初始化tinc操作
    let mut tinc = Tinc::new(
        settings.tinc.home_path.clone(),
        settings.tinc.pub_key_path.clone()
    );

    // 监测tinc pub key 不存在或生成时间超过一个月，将生成tinc pub key
    info!("check_pub_key");
    if !check_pub_key(&settings.tinc.home_path, &settings.tinc.pub_key_path) {
        tinc.create_pub_key();
    }

    // 获取本地 tinc geo 和 ip信息，创建proxy uuid
    info!("Get local info.");
    let mut info = match Info::new_from_local(&settings) {
        Ok(x) => x,
        Err(e) => {
            error!("{}", e.description());
            std::process::exit(1);
        }
    };
    info.proxy_info.create_uid();

    // 初始化上报操作
    let client = Client::new(settings.server.url.clone());
    debug!("{:?}",client);
    // Client Login
    {
        if client.proxy_login(&settings, &mut info) {
            if !tinc.write_auth_file(
                &settings.server.url,
                &info,
            ) {
                error!("Write auth file failed");
                std::process::exit(1);
            }
        }
        else {
            error!("Proxy login failed");
            std::process::exit(1);
        };
    }

    // 注册proxy
    info!("proxy_register");
    if !client.proxy_register(&mut info) {
        error!("Proxy register failed");
        std::process::exit(1);
    }

    // 初次上传heartbeat
    info!("proxy_heart_beat");
    if !client.proxy_heart_beat(&info) {
        error!("Proxy heart beat send failed");
        std::process::exit(1);
    };

    // 初次获取其他proxy信息
    info!("proxy_get_online_proxy");
    if !client.proxy_get_online_proxy(&mut info) {
        error!("proxy get online proxy failed");
        std::process::exit(1);
    };

    if !tinc.check_info(&info) {
        error!("proxy check online proxy  info failed");
        std::process::exit(1);
    }

    // 添加多线程 同步操作锁
    // 目前client，仅用于main loop，上传心跳
    let client_arc = Arc::new(Mutex::new(client));

    // tinc操作 main loop：监测tinc运行，修改pub key
    // web_server：添加hosts
    let tinc_arc = Arc::new(Mutex::new(tinc));

    // 信息包括 geo信息：初次启动获取，目前初始化后无更新
    //          tinc信息： 本机tinc运行参数
    //          proxy信息：公网ip， uuid等
    //          目前 初始化后 main loop 和web_server 都只做读取
    let info_arc = Arc::new(Mutex::new(info));

    let info_arc_clone = info_arc.clone();
    let tinc_arc_clone = tinc_arc.clone();

    // 启动web_server,线程
    thread::spawn(move ||web_server(info_arc_clone, tinc_arc_clone));

    let mut daemon = Daemon::new(tinc_arc, client_arc, info_arc, settings);
    daemon.run();

}

#[derive(Clone, Debug)]
enum DaemonEvent {
    Schedule(ScheduleType),
    Heartbeat(Status),
    TincCheck(Status),
    OnlineProxy(Status),
}

#[derive(Clone, Debug)]
enum ScheduleType {
    Heartbeat,
    TincCheck,
    OnlineProxy,
}

struct DaemonStatus {
    heartbeat_status:   Status,
    tinccheck_status:   Status,
    onlineproxy_status: Status,
}
impl DaemonStatus {
    fn new() -> Self {
        DaemonStatus {
            heartbeat_status:   Status::Finish,
            tinccheck_status:   Status::Finish,
            onlineproxy_status: Status::Finish,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Status {
    Execute,
    Finish,
    Error,
}

struct Daemon {
    tinc_arc:               Arc<Mutex<Tinc>>,
    client_arc:             Arc<Mutex<Client>>,
    info_arc:               Arc<Mutex<Info>>,
    settings:               Settings,
    daemon_event_tx:        mpsc::Sender<DaemonEvent>,
    daemon_event_rx:        mpsc::Receiver<DaemonEvent>,
    daemon_status:          DaemonStatus,
}

impl Daemon {
    fn new(
        tinc_arc: Arc<Mutex<Tinc>>,
        client_arc: Arc<Mutex<Client>>,
        info_arc: Arc<Mutex<Info>>,
        settings: Settings,
    ) -> Self {
        let (daemon_event_tx, daemon_event_rx) = mpsc::channel();
        let daemon_status = DaemonStatus::new();
        Daemon {
            tinc_arc,
            client_arc,
            info_arc,
            settings,
            daemon_event_tx,
            daemon_event_rx,
            daemon_status,
        }
    }

    fn run(&mut self) {
        let daemon_event_tx_clone = self.daemon_event_tx.clone();
        thread::spawn(move || schedule(daemon_event_tx_clone));
        while let Ok(event) = self.daemon_event_rx.recv() {
            self.handle_event(event);
        }
    }

    fn handle_event(&mut self, event: DaemonEvent) {
        match event {
            DaemonEvent::Schedule(schedule) => self.exec_schedule(schedule),
            DaemonEvent::Heartbeat(status) => {
                self.daemon_status.heartbeat_status = status.clone();
                if status == Status::Error {
                    self.exec_schedule(ScheduleType::Heartbeat);
                }
            },
            DaemonEvent::TincCheck(status) => {
                self.daemon_status.tinccheck_status = status.clone();
                if status == Status::Error {
                    self.exec_schedule(ScheduleType::TincCheck);
                }
            },
            DaemonEvent::OnlineProxy(status) => {
                self.daemon_status.onlineproxy_status = status.clone();
                if status == Status::Error {
                    self.exec_schedule(ScheduleType::OnlineProxy);
                }
            },
        };
    }

    fn exec_schedule(&mut self, schedule: ScheduleType) {
        let daemon_event_tx = self.daemon_event_tx.clone();
        match schedule {
            ScheduleType::Heartbeat => {
                let client_arc_clone = self.client_arc.clone();
                let info_arc_clone = self.info_arc.clone();
                if self.daemon_status.heartbeat_status != Status::Execute {
                    self.daemon_status.heartbeat_status = Status::Execute;
                    thread::spawn(move || exec_heartbeat(
                        client_arc_clone,
                        info_arc_clone,
                        daemon_event_tx,
                    ));
                }
            }
            ScheduleType::TincCheck => {
                let tinc_arc_clone = self.tinc_arc.clone();
                let tinc_home = self.settings.tinc.home_path.clone();
                if self.daemon_status.tinccheck_status != Status::Execute {
                    self.daemon_status.tinccheck_status = Status::Execute;
                    thread::spawn(move || exec_tinc_check(
                        tinc_arc_clone,
                        daemon_event_tx,
                        tinc_home,
                    ));
                }
            }
            ScheduleType::OnlineProxy => {
                let client_arc_clone = self.client_arc.clone();
                let tinc_arc_clone = self.tinc_arc.clone();
                let info_arc_clone = self.info_arc.clone();
                if self.daemon_status.onlineproxy_status != Status::Execute {
                    self.daemon_status.onlineproxy_status = Status::Execute;
                    thread::spawn(move || exec_online_proxy(
                        client_arc_clone,
                        info_arc_clone,
                        tinc_arc_clone,
                        daemon_event_tx,
                    ));
                }
            }
        }
    }
}

fn schedule(
    daemon_event_tx: mpsc::Sender<DaemonEvent>
) {
    let daemon_event_tx = daemon_event_tx;
    let heartbeat_frequency = Duration::from_secs(20);
    let check_tinc_frequency = Duration::from_secs(3);
    let get_online_proxy_frequency = Duration::from_secs(10);

    // now: 当前时间
    // heartbeat_time：前次心跳发送时间
    // check_tinc_time： 前次监测tinc时间
    let mut now = Instant::now();
    let mut heartbeat_time = now.clone();
    let mut check_tinc_time = now.clone();
    let mut get_online_proxy_time = now.clone();
    loop {
        if now.duration_since(heartbeat_time) > heartbeat_frequency {
            let _ = daemon_event_tx.send(DaemonEvent::Schedule(ScheduleType::Heartbeat));
            heartbeat_time = now.clone();
        }
        if now.duration_since(check_tinc_time) > check_tinc_frequency {
            let _ = daemon_event_tx.send(DaemonEvent::Schedule(ScheduleType::TincCheck));
            check_tinc_time = now.clone();
        }
        if now.duration_since(get_online_proxy_time) > get_online_proxy_frequency {
            let _ = daemon_event_tx.send(DaemonEvent::Schedule(ScheduleType::OnlineProxy));
            get_online_proxy_time = now.clone();
        }
        sleep(Duration::from_millis(500));
        now = Instant::now();
    }
}

fn exec_heartbeat(
    client_arc:                 Arc<Mutex<Client>>,
    info_arc:                   Arc<Mutex<Info>>,
    daemon_event_tx:            mpsc::Sender<DaemonEvent>,
) {
    info!("proxy_heart_beat");
    if let Ok(client) = client_arc.try_lock() {
        if let Ok(info) = info_arc.try_lock() {
            if !client.proxy_heart_beat(&info) {
                error!("Heart beat send failed.")
            } else {
                let _ = daemon_event_tx.send(DaemonEvent::Heartbeat(Status::Finish));
                return;
            }
        }
    }
    sleep(Duration::from_millis(500));
    let _ = daemon_event_tx.send(DaemonEvent::Heartbeat(Status::Error));
}

fn exec_tinc_check(
    tinc_arc:                   Arc<Mutex<Tinc>>,
    daemon_event_tx:            mpsc::Sender<DaemonEvent>,
    tinc_home:                  String,
) {
    info!("check_tinc_status");
    if check_tinc_status(&tinc_home) {
        let _ = daemon_event_tx.send(DaemonEvent::TincCheck(Status::Finish));
        return;
    } else {
        if let Ok(mut tinc) = tinc_arc.try_lock() {
            tinc.restart_tinc();
            if check_tinc_status(&tinc_home) {
                let _ = daemon_event_tx.send(DaemonEvent::TincCheck(Status::Finish));
                return;
            } else {
                error!("Restart tinc failed");
            }
        }
    }
    sleep(Duration::from_millis(500));
    let _ = daemon_event_tx.send(DaemonEvent::TincCheck(Status::Error));
}

fn exec_online_proxy(
    client_arc:                 Arc<Mutex<Client>>,
    info_arc:                   Arc<Mutex<Info>>,
    tinc_arc:                   Arc<Mutex<Tinc>>,
    daemon_event_tx:        mpsc::Sender<DaemonEvent>,
) {
    info!("exec_online_proxy");
    if let Ok(client) = client_arc.try_lock() {
        if let Ok(mut info) = info_arc.try_lock() {
            if let Ok(mut tinc) = tinc_arc.try_lock() {
                if client.proxy_get_online_proxy(&mut info) {
                    if tinc.check_info(&mut info) {
                        let _ = daemon_event_tx.send(DaemonEvent::OnlineProxy(Status::Finish));
                        return;
                    } else {
                        error!("Tinc check_info failed.");
                    }
                } else {
                    error!("proxy_get_online_proxy failed.");
                }
            }
        }
    }
    sleep(Duration::from_millis(500));
    let _ = daemon_event_tx.send(DaemonEvent::OnlineProxy(Status::Error));
}