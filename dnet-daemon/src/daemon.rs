use std::sync::{Arc, Mutex, mpsc};

use serde_json::to_value;

use crate::traits::{InfoTrait, RpcTrait, TunnelTrait};
use crate::cmd_api::types::{IpcCommand, CommandResponse};
use crate::info::Info;
use crate::http_server_client::RpcMonitor;
use crate::tinc_manager::TincMonitor;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum RpcState {
    Connecting,
    Connected,
    ReConnecting,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum TunnelState {
    Connecting,
    Connected,
    DisConnecting,
    DisConnected,
    TunnelInitFailed(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum DaemonExecutionState {
    Running,
    Finished,
}

pub enum TunnelCommand {
    Connect,
    Disconnect,
}

#[derive(Clone, Debug)]
pub enum DaemonEvent {
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

    IpcCommand(IpcCommand),

    // Ctrl + c && kill
    ShutDown,
}

pub struct Daemon {
    info_arc:               Arc<Mutex<Info>>,
    daemon_event_tx:        mpsc::Sender<DaemonEvent>,
    daemon_event_rx:        mpsc::Receiver<DaemonEvent>,
    status:                 State,
    tunnel_command_tx:      mpsc::Sender<TunnelCommand>,
}

impl Daemon {
    pub fn start() -> Self {
        let (daemon_event_tx, daemon_event_rx) = mpsc::channel();

        let _ = crate::set_shutdown_signal_handler(daemon_event_tx.clone());

        let (tinc, tunnel_command_tx) = TincMonitor::new(daemon_event_tx.clone());
        tinc.start_monitor();

        info!("Get local info.");
        let mut info = Info::new(daemon_event_tx.clone());
        info.create_uid();

        let info_arc = Arc::new(Mutex::new(info));

        let mut _rpc = RpcMonitor::new(info_arc.clone(), daemon_event_tx.clone());
        _rpc.start_monitor();

        Daemon {
            info_arc,
            daemon_event_tx,
            daemon_event_rx,
            status: State::new(),
            tunnel_command_tx,
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
            DaemonEvent::IpcCommand(cmd) => {
                self.handle_ipc_command_event(cmd);
            }
            // Ctrl + c && kill
            DaemonEvent::ShutDown => {
                self.handle_shutdown();
            },
        };
    }

    fn handle_shutdown(&mut self) {
        self.status.daemon = DaemonExecutionState::Finished;
    }

    fn handle_ipc_command_event(&mut self, cmd: IpcCommand) {
        match cmd {
            IpcCommand::TunnelConnect(tx) => {
                self.tunnel_command_tx.send(TunnelCommand::Connect);
                tx.send(CommandResponse::success());
            }
            IpcCommand::TunnelDisConnect(tx) => {
                self.tunnel_command_tx.send(TunnelCommand::Disconnect);
                tx.send(CommandResponse::success());
            }
            IpcCommand::TunnelStatus(tx) => {
                let mut response = CommandResponse::success();
                response.data = Some(to_value(&self.status.tunnel).unwrap());
                tx.send(response);
            }
            IpcCommand::RpcStatus(tx) => {
                let mut response = CommandResponse::success();
                response.data = Some(to_value(&self.status.rpc).unwrap());
                tx.send(response);
            }
            IpcCommand::GroupInfo(tx, id) => {
                tx.send(CommandResponse::success());
            }
        }
    }
}
