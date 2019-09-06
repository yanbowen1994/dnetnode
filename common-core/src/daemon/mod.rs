use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use crate::get_settings;
use crate::traits::{InfoTrait, RpcTrait, TunnelTrait};

//pub type Result<T> = std::result::Result<T, Error>;
//
//#[derive(err_derive::Error, Debug)]
//pub enum Error {
//    #[error(display = "Get local info")]
//    GetLocalInfo(#[error(cause)] ::domain::Error),

//    #[error(display = "Conductor connect failed.")]
//    ConductorConnect(#[error(cause)] ::http_server_client::client::Error),

//    #[error(display = "Tinc operator error.")]
//    TincOperator(#[error(cause)] TincOperatorError),
//}

struct State {
    rpc:        RpcState,
    tunnel:     TunnelState,
    daemon:     DaemonExecutionState,
}

impl State {
    fn new() -> Self {
        State {
            rpc:    RpcState::Connecting,
            tunnel: TunnelState::DisConnected,
            daemon: DaemonExecutionState::Running,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RpcState {
    Connecting,
    Connected,
    ReConnecting,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TunnelState {
    Connecting,
    Connected,
    DisConnecting,
    DisConnected,
    TunnelInitFailed(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DaemonExecutionState {
    Running,
    Finished,
}

#[derive(Clone, Debug)]
pub enum DaemonEvent
{
    // -> self.Status.rpc.Connected
    RpcConnected,

    // if init -> self.Status.rpc.Connecting
    // else -> self.Status.rpc.ReConnecting
    RpcConnecting,

    // -> self.Status.tunnel.Connected
    TunnelConnected,

    // -> self.Status.tunnel.DisConnected
    TunnelDisConnected,

    // ->
    TunnelInitFailed(String),

    // Ctrl + c && kill
    ShutDown,
}

pub struct Daemon<Info>
    where Info: InfoTrait,
{
    info_arc:               Arc<Mutex<Info>>,
    daemon_event_tx:        mpsc::Sender<DaemonEvent>,
    daemon_event_rx:        mpsc::Receiver<DaemonEvent>,
    status:                 State,
}

impl<Info> Daemon<Info>
    where Info: InfoTrait,
{
    pub fn start<Rpc, Tunnel>() -> Self
        where Rpc: RpcTrait<Info>,
              Tunnel: TunnelTrait,
    {
        let (daemon_event_tx, daemon_event_rx) = mpsc::channel();

        let _ = crate::set_shutdown_signal_handler(daemon_event_tx.clone());

        // tinc操作 main loop：监测tinc运行，修改pub key
        // web_server：添加hosts
        let mut tinc = Tunnel::new(daemon_event_tx.clone());
        tinc.start_monitor();

        // 获取本地 tinc geo 和 ip信息，创建proxy uuid
        info!("Get local info.");
        let mut info = Info::new(daemon_event_tx.clone());
        info.create_uid();

        // 信息包括 geo信息：初次启动获取，目前初始化后无更新
        //          tinc信息： 本机tinc运行参数
        //          proxy信息：公网ip， uuid等
        //          目前 初始化后 main loop 和web_server 都只做读取
        let info_arc = Arc::new(Mutex::new(info));

        let mut rpc = Rpc::new(info_arc.clone(), daemon_event_tx.clone());
        rpc.start_monitor();

        Daemon {
            info_arc,
            daemon_event_tx,
            daemon_event_rx,
            status: State::new(),
        }
    }

    pub fn run(&mut self) {
        while let Ok(event) = self.daemon_event_rx.recv() {
            self.handle_event(event);
            if self.status.daemon == DaemonExecutionState::Finished {
                break;
            }
        }
    }

    fn handle_event(&mut self, event: DaemonEvent) {
        match event {
            // status change
            DaemonEvent::RpcConnected => {
                self.status.rpc = RpcState::Connected;
            },
            DaemonEvent::RpcConnecting => {
                if RpcState::Connecting != self.status.rpc {
                    self.status.rpc = RpcState::ReConnecting;
                }
            },
            DaemonEvent::TunnelConnected => {
                self.status.tunnel = TunnelState::Connected;
            },
            DaemonEvent::TunnelDisConnected => {
                self.status.tunnel = TunnelState::DisConnected;
            },
            DaemonEvent::TunnelInitFailed(err_str) => {
                self.status.tunnel = TunnelState::TunnelInitFailed(err_str);
            },
            // Ctrl + c && kill
            DaemonEvent::ShutDown => {
                self.handle_shutdown();
            },
        };
    }

    fn handle_shutdown(&mut self) {
        self.status.daemon = DaemonExecutionState::Finished;
    }
}
