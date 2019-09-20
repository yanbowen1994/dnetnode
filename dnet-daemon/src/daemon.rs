use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use futures::sync::oneshot;

use dnet_types::states::{DaemonExecutionState, TunnelState, State, RpcState};

use crate::traits::{InfoTrait, RpcTrait, TunnelTrait};
use crate::info::{self, Info};
use crate::rpc::{self, RpcMonitor};
use crate::tinc_manager::{TincMonitor, TincOperator};
use crate::cmd_api::ipc_server::{ManagementInterfaceServer, ManagementCommand, ManagementInterfaceEventBroadcaster};
use crate::mpsc::IntoSender;
use crate::settings::get_settings;
use dnet_types::settings::RunMode;
use dnet_types::response::Response;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Tinc can't supported ipv6")]
    UnsupportedTunnel,

    /// Error in the management interface
    #[error(display = "Unable to start management interface server")]
    StartManagementInterface(#[error(cause)] talpid_ipc::Error),

    #[error(display = "Unable to start management interface server")]
    InfoError(#[error(cause)] info::Error),

    #[error(display = "Tunnel init failed.")]
    TunnelInit(#[error(cause)] tinc_plugin::TincOperatorError),
}

pub enum TunnelCommand {
    Connect,
    Disconnect,
}

pub enum DaemonEvent {
    // -> self.Status.rpc.Connected
    RpcConnected,

    // if init -> self.Status.rpc.Connecting
    // else -> self.Status.rpc.ReConnecting
    RpcConnecting,

    // -> self.Status.tunnel.Connected
    TunnelConnected,

    // -> self.Status.tunnel.Disconnected
    TunnelDisconnected,

    // ->
    TunnelInitFailed(String),

    ManagementCommand(ManagementCommand),

    // Ctrl + c && kill
    ShutDown,
}

impl From<ManagementCommand> for DaemonEvent {
    fn from(command: ManagementCommand) -> Self {
        DaemonEvent::ManagementCommand(command)
    }
}

pub struct Daemon {
    daemon_event_tx:        mpsc::Sender<DaemonEvent>,
    daemon_event_rx:        mpsc::Receiver<DaemonEvent>,
    status:                 State,
    tunnel_command_tx:      mpsc::Sender<TunnelCommand>,
}

impl Daemon {
    pub fn start() -> Result<Self> {
        let (daemon_event_tx, daemon_event_rx) = mpsc::channel();

        let _ = crate::set_shutdown_signal_handler(daemon_event_tx.clone());

        Self::start_management_interface(daemon_event_tx.clone())?;

        TincOperator::new().init()
            .map_err(Error::TunnelInit)?;

        info!("Init local info.");
        Info::new().map_err(Error::InfoError)?;

        let run_mode = &get_settings().common.mode;
        if run_mode == &RunMode::Proxy {
            let mut _rpc = RpcMonitor::<rpc::proxy::RpcMonitor>::new(daemon_event_tx.clone());
            _rpc.start_monitor();
        }
        else {
            let mut _rpc = RpcMonitor::<rpc::client::RpcMonitor>::new(daemon_event_tx.clone());
            _rpc.start_monitor();
        }

        let (tinc, tunnel_command_tx) =
            TincMonitor::new(daemon_event_tx.clone());
        tinc.start_monitor();

        Ok(Daemon {
            daemon_event_tx,
            daemon_event_rx,
            status: State::new(),
            tunnel_command_tx,
        })
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
                self.handle_rpc_connected();
            },
            DaemonEvent::RpcConnecting => {
                if RpcState::Connecting != self.status.rpc {
                    self.status.rpc = RpcState::ReConnecting;
                }
            },
            DaemonEvent::TunnelConnected => {
                self.status.tunnel = TunnelState::Connected;
            },
            DaemonEvent::TunnelDisconnected => {
                self.status.tunnel = TunnelState::Disconnected;
            },
            DaemonEvent::TunnelInitFailed(err_str) => {
                self.status.tunnel = TunnelState::TunnelInitFailed(err_str);
            },
            DaemonEvent::ManagementCommand(cmd) => {
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

    fn handle_ipc_command_event(&mut self, cmd: ManagementCommand) {
        match cmd {
            ManagementCommand::TunnelConnect(tx) => {
                let _ = self.tunnel_command_tx.send(TunnelCommand::Connect);
                // TODO CommandResponse
//                Self::oneshot_send(tx, Box::new(CommandResponse::success()), "");
                let _ = Self::oneshot_send(tx, (), "");
            }

            ManagementCommand::TunnelDisconnect(tx) => {
                let _ = self.tunnel_command_tx.send(TunnelCommand::Disconnect);
                let _ = Self::oneshot_send(tx, (), "");
            }

            ManagementCommand::State(tx) => {
//                let mut response = CommandResponse::success();
//                response.data = Some(to_value(&self.status.rpc).unwrap());
                let state = self.status.clone();
                let _ = Self::oneshot_send(tx, state, "");
            }

            ManagementCommand::GroupInfo(tx, id) => {
                let _ = Self::oneshot_send(tx, (), "");
            }

            ManagementCommand::Shutdown(tx) => {
                let _ = self.daemon_event_tx.send(DaemonEvent::ShutDown);

                let command_response = Response::success();

                info!("Shutdown by cli command.");

                let _ = Self::oneshot_send(tx, command_response, "");
            }
        }
    }

    fn handle_rpc_connected(&mut self) {
        self.status.rpc = RpcState::Connected;
        let run_mode = get_settings().common.mode.clone();
        if run_mode == RunMode::Proxy {
            self.tunnel_command_tx.send(TunnelCommand::Connect).unwrap();
        }
        else {
            let auto_connect = get_settings().client.auto_connect;
            if auto_connect == true {
                if self.status.tunnel == TunnelState::Disconnected ||
                    self.status.tunnel == TunnelState::Disconnecting {
                    self.tunnel_command_tx.send(TunnelCommand::Connect).unwrap();
                }
            }
        }
    }

    fn oneshot_send<T>(tx: oneshot::Sender<T>, t: T, msg: &'static str) {
        if tx.send(t).is_err() {
            warn!("Unable to send {} to management interface client", msg);
        }
    }

    // Starts the management interface and spawns a thread that will process it.
    // Returns a handle that allows notifying all subscribers on events.
    fn start_management_interface(
        event_tx: mpsc::Sender<DaemonEvent>,
    ) -> Result<ManagementInterfaceEventBroadcaster> {
        let multiplex_event_tx = IntoSender::from(event_tx.clone());
        let server = Self::start_management_interface_server(multiplex_event_tx)?;
        let event_broadcaster = server.event_broadcaster();
        Self::spawn_management_interface_wait_thread(server, event_tx);
        Ok(event_broadcaster)
    }

    fn start_management_interface_server(
        event_tx: IntoSender<ManagementCommand, DaemonEvent>,
    ) -> Result<ManagementInterfaceServer> {
        // TODO ipc path
        let server =
            ManagementInterfaceServer::start("/opt/dnet/dnet.socket", event_tx).map_err(Error::StartManagementInterface)?;
        info!("Management interface listening on {}", server.socket_path());

        Ok(server)
    }

    fn spawn_management_interface_wait_thread(
        server: ManagementInterfaceServer,
        exit_tx: mpsc::Sender<DaemonEvent>,
    ) {
        thread::spawn(move || {
            server.wait();
            info!("Management interface shut down");
//            let _ = exit_tx.send(DaemonEvent::ManagementInterfaceExited);
        });
    }
}
