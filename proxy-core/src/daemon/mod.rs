use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use common_core::get_settings;
use tinc_manager::check::*;
use tinc_manager::TincOperator;
use domain::Info;
use http_server_client::Client;
use http_server_client::web_server;
use tinc_plugin::TincOperatorError;

mod rpc_monitor;
mod tinc_monitor;
mod shutdown;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Get local info")]
    GetLocalInfo(#[error(cause)] ::domain::Error),

    #[error(display = "Conductor connect failed.")]
    ConductorConnect(#[error(cause)] ::http_server_client::client::Error),

    #[error(display = "Tinc operator error.")]
    TincOperator(#[error(cause)] TincOperatorError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DaemonExecutionState {
    Running,
    Finished,
}

#[derive(Clone, Debug)]
pub enum DaemonEvent {
    RpcFailed,
//    TincNotExist,
    RpcRestart(Arc<Mutex<Client>>),
    ShutDown,
}

pub struct Daemon {
    client_arc:             Arc<Mutex<Client>>,
    info_arc:               Arc<Mutex<Info>>,
    daemon_event_tx:        mpsc::Sender<DaemonEvent>,
    daemon_event_rx:        mpsc::Receiver<DaemonEvent>,
    status:                 DaemonExecutionState,
}

impl Daemon {
    pub fn start() -> Result<Self> {
        // tinc操作 main loop：监测tinc运行，修改pub key
        // web_server：添加hosts
        let tinc = Daemon::init_tinc()?;

        // 获取本地 tinc geo 和 ip信息，创建proxy uuid
        info!("Get local info.");
        let mut info = Info::new_from_local()
            .map_err(Error::GetLocalInfo)?;
        info.proxy_info.create_uid();

        tinc.set_info_to_local(&mut info)
            .map_err(Error::TincOperator)?;

        let (daemon_event_tx, daemon_event_rx) = mpsc::channel();

        let client = Daemon::init_client(&mut info);

        // 信息包括 geo信息：初次启动获取，目前初始化后无更新
        //          tinc信息： 本机tinc运行参数
        //          proxy信息：公网ip， uuid等
        //          目前 初始化后 main loop 和web_server 都只做读取
        let info_arc = Arc::new(Mutex::new(info));

        // 添加多线程 同步操作锁
        // 目前client，仅用于main loop，上传心跳
        let client_arc = Arc::new(Mutex::new(client));

        let _ = shutdown::set_shutdown_signal_handler(daemon_event_tx.clone());

        let info_arc_clone = info_arc.clone();

        let web_server_tx = daemon_event_tx.clone();
        // 启动web_server,线程
        thread::spawn(move ||
            web_server(info_arc_clone,
                       Arc::new(Mutex::new(
                           TincOperator::new())),
                       web_server_tx,
            )
        );
        rpc_monitor::spawn(
            client_arc.clone(),
            info_arc.clone(),
            daemon_event_tx.clone(),
        );
        tinc_monitor::TincMonitor::new(tinc)
            .map_err(Error::TincOperator)?
            .spawn();

        Ok(Daemon {
            client_arc,
            info_arc,
            daemon_event_tx,
            daemon_event_rx,
            status: DaemonExecutionState::Running,
        })
    }

    pub fn run(&mut self) {
        while let Ok(event) = self.daemon_event_rx.recv() {
            self.handle_event(event);
            if self.status == DaemonExecutionState::Finished {
                break;
            }
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
            DaemonEvent::ShutDown => {
                self.handle_shutdown();
            }
        };
    }

    fn handle_rpc_failed(&self) {
        let daemon_event_tx = self.daemon_event_tx.clone();
        let info_arc = self.info_arc.clone();
        thread::spawn(move||Daemon::reconnect_to_conductor(
            daemon_event_tx,
            info_arc,
        ));
    }

    fn handle_rpc_restart(&mut self,
                          client_arc: Arc<Mutex<Client>>,
    ) {
        rpc_monitor::spawn(
            client_arc.clone(),
            self.info_arc.clone(),
            self.daemon_event_tx.clone(),
        );
        self.client_arc = client_arc;
    }

    fn handle_shutdown(&mut self) {
        self.status = DaemonExecutionState::Finished;
    }

    fn reconnect_to_conductor(
        daemon_event_tx:    mpsc::Sender<DaemonEvent>,
        info_arc:           Arc<Mutex<Info>>,
    ) {
        if let Ok(mut info) = info_arc.lock() {
            let client = Daemon::init_client(&mut info);
            let client_arc = Arc::new(Mutex::new(client));
            loop {
                if let Ok(_) = daemon_event_tx.send(
                    DaemonEvent::RpcRestart(client_arc.clone())) {
                    break;
                }
            }
        }
    }

    fn init_tinc() -> Result<TincOperator> {
        // 初始化tinc操作
        let tinc = TincOperator::new();

        tinc.create_tinc_dirs()
            .map_err(Error::TincOperator)?;

        // 监测tinc pub key 不存在或生成时间超过一个月，将生成tinc pub key
        info!("check_pub_key");
        if !check_pub_key() {
            tinc.create_pub_key()
                .map_err(Error::TincOperator)?;
        }
        Ok(tinc)
    }

    fn init_client(
        info: &mut Info,
    ) -> Client {
        let settings = get_settings();

        let tinc = TincOperator::new();

        // 初始化上报操作
        let client = Client::new();
        loop {
            // Client Login
            {
                if let Err(e) = client.proxy_login(&settings, info).map_err(Error::ConductorConnect) {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }

                if let Err(e) = tinc.write_auth_file(&settings.server.url, info)
                    .map_err(Error::TincOperator) {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            // 注册proxy
            info!("proxy_register");
            {
                if let Err(e) = client.proxy_register(info).map_err(Error::ConductorConnect) {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            // 初次上传heartbeat
            info!("proxy_heart_beat");
            {
                if let Err(e) = client.proxy_heart_beat(info).map_err(Error::ConductorConnect) {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }
            return client;
        }
    }
}
