use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use settings::Settings;
use tinc_manager::check::*;
use tinc_manager::TincOperator;
use domain::Info;
use http_server_client::Client;
use http_server_client::web_server;

mod rpc_monitor;
mod tinc_monitor;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Get local info")]
    GetLocalInfo(#[error(cause)] ::domain::Error),

    #[error(display = "Conductor connect failed.")]
    ConductorConnect(#[error(cause)] ::http_server_client::client::Error),

    #[error(display = "Tinc operator error.")]
    TincOperator(#[error(cause)] ::tinc_manager::Error),
}

#[derive(Clone, Debug)]
pub enum DaemonEvent {
    RpcFailed,
//    TincNotExist,
    RpcRestart(Arc<Mutex<Client>>),
}

pub struct Daemon {
    settings:               Settings,
    client_arc:             Arc<Mutex<Client>>,
    info_arc:               Arc<Mutex<Info>>,
    daemon_event_tx:        mpsc::Sender<DaemonEvent>,
    daemon_event_rx:        mpsc::Receiver<DaemonEvent>,
}

impl Daemon {
    pub fn start(
        settings: Settings,
    ) -> Result<Self> {
        // 获取本地 tinc geo 和 ip信息，创建proxy uuid
        info!("Get local info.");
        let mut info = Info::new_from_local(&settings)
            .map_err(Error::GetLocalInfo)?;
        info.proxy_info.create_uid();

        let client = Daemon::init_client(&settings, &mut info)?;

        // 信息包括 geo信息：初次启动获取，目前初始化后无更新
        //          tinc信息： 本机tinc运行参数
        //          proxy信息：公网ip， uuid等
        //          目前 初始化后 main loop 和web_server 都只做读取
        let info_arc = Arc::new(Mutex::new(info));

        // tinc操作 main loop：监测tinc运行，修改pub key
        // web_server：添加hosts
        let tinc = Daemon::init_tinc(&settings)?;

        // 添加多线程 同步操作锁
        // 目前client，仅用于main loop，上传心跳
        let client_arc = Arc::new(Mutex::new(client));

        let (daemon_event_tx, daemon_event_rx) = mpsc::channel();

        let info_arc_clone = info_arc.clone();

        // 启动web_server,线程
        let tinc_home_path = settings.tinc.home_path.clone();
        let server_port = settings.client.server_port.clone();
        thread::spawn(move ||
            web_server(info_arc_clone,
                       Arc::new(Mutex::new(
                           TincOperator::new(tinc_home_path))),
                       &server_port));
        rpc_monitor::spawn(
            client_arc.clone(),
            info_arc.clone(),
            settings.tinc.home_path.clone(),
            daemon_event_tx.clone(),
        );
        tinc_monitor::TincMonitor::new(tinc)
            .map_err(Error::TincOperator)?
            .spawn();

        Ok(Daemon {
            settings,
            client_arc,
            info_arc,
            daemon_event_tx,
            daemon_event_rx,
        })
    }

    pub fn run(&mut self) {
        while let Ok(event) = self.daemon_event_rx.recv() {
            self.handle_event(event);
        }
    }

    fn handle_event(&mut self, event: DaemonEvent) {
        match event {
            DaemonEvent::RpcFailed => {
                self.handle_rpc_failed();
            },
            DaemonEvent::RpcRestart(client_arc) => {
                self.handle_rpc_restart(client_arc);
            }
        };
    }

    fn handle_rpc_failed(&self) {
        let settings = self.settings.clone();
        let daemon_event_tx = self.daemon_event_tx.clone();
        let info_arc = self.info_arc.clone();
        thread::spawn(move||Daemon::reconnect_to_conductor(
            daemon_event_tx,
            settings,
            info_arc,
        ));
    }

    fn handle_rpc_restart(&mut self,
                          client_arc: Arc<Mutex<Client>>,
    ) {
        rpc_monitor::spawn(
            client_arc.clone(),
            self.info_arc.clone(),
            self.settings.tinc.home_path.clone(),
            self.daemon_event_tx.clone(),
        );
        self.client_arc = client_arc;
    }

    fn reconnect_to_conductor(
        daemon_event_tx:    mpsc::Sender<DaemonEvent>,
        settings:           Settings,
        info_arc:           Arc<Mutex<Info>>,
    ) {
        loop {
            if let Ok(mut info) = info_arc.lock() {
                let client = match Daemon::init_client(
                    &settings,
                    &mut info
                ) {
                    Ok(x) => x,
                    Err(e) => {
                        error!("{:#?}", e);
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    },
                };

                let client_arc = Arc::new(Mutex::new(client));
                if let Ok(_) = daemon_event_tx.send(DaemonEvent::RpcRestart(client_arc)) {
                    break;
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
    }

    fn init_tinc(settings: &Settings) -> Result<TincOperator> {
        // 初始化tinc操作
        let tinc = TincOperator::new(
            settings.tinc.home_path.clone()
        );

        // 监测tinc pub key 不存在或生成时间超过一个月，将生成tinc pub key
        info!("check_pub_key");
        if !check_pub_key(&settings.tinc.home_path, &settings.tinc.pub_key_path) {
            tinc.create_pub_key()
                .map_err(Error::TincOperator)?;
        }
        Ok(tinc)
    }

    fn init_client(
        settings: &Settings,
        info: &mut Info,
    ) -> Result<Client> {
        let tinc = TincOperator::new(settings.tinc.home_path.clone());

        // 初始化上报操作
        let client = Client::new(settings.server.url.clone());
        debug!("{:?}",client);
        // Client Login
        {
            client.proxy_login(&settings, info).map_err(Error::ConductorConnect)?;
            tinc.write_auth_file(&settings.server.url, info)
                .map_err(Error::TincOperator)?;
        }

        // 注册proxy
        info!("proxy_register");
        {
            client.proxy_register(info).map_err(Error::ConductorConnect)?;
        }

        // 初次上传heartbeat
        info!("proxy_heart_beat");
        {
            client.proxy_heart_beat(&info).map_err(Error::ConductorConnect)?;
        }

        // 初次获取其他proxy信息
        info!("proxy_get_online_proxy");
        client.proxy_get_online_proxy(info, &settings.tinc.home_path).map_err(Error::ConductorConnect)?;
        return Ok(client);
    }
}
