use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use futures::sync::oneshot;

use dnet_types::states::{DaemonExecutionState, TunnelState, State, RpcState};

use crate::traits::{InfoTrait, RpcTrait, TunnelTrait};
use crate::info::Info;
use crate::http_server_client::RpcMonitor;
use crate::tinc_manager::TincMonitor;
use crate::cmd_api::ipc_server::{ManagementInterfaceServer, ManagementCommand, ManagementInterfaceEventBroadcaster};
use crate::mpsc::IntoSender;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Tinc can't supported ipv6")]
    UnsupportedTunnel,

    /// Error in the management interface
    #[error(display = "Unable to start management interface server")]
    StartManagementInterface(#[error(cause)] talpid_ipc::Error),
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

    // -> self.Status.tunnel.DisConnected
    TunnelDisConnected,

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

        Self::start_management_interface(daemon_event_tx.clone());

        info!("Get local info.");
        let mut info = Info::new(daemon_event_tx.clone());
        info.create_uid();

        let info_arc = Arc::new(Mutex::new(info));

        let (tinc, tunnel_command_tx) =
            TincMonitor::new(daemon_event_tx.clone(), info_arc.clone());
        tinc.start_monitor();

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
                self.tunnel_command_tx.send(TunnelCommand::Connect);
                // TODO CommandResponse
//                Self::oneshot_send(tx, Box::new(CommandResponse::success()), "");
                Self::oneshot_send(tx, (), "");
            }

            ManagementCommand::TunnelDisConnect(tx) => {
                self.tunnel_command_tx.send(TunnelCommand::Disconnect);
                Self::oneshot_send(tx, (), "");
            }

            ManagementCommand::State(tx) => {
//                let mut response = CommandResponse::success();
//                response.data = Some(to_value(&self.status.rpc).unwrap());
                let state = self.status.clone();
                Self::oneshot_send(tx, state, "");
            }

            ManagementCommand::GroupInfo(tx, id) => {
                Self::oneshot_send(tx, (), "");
            }

            ManagementCommand::Shutdown => {
//                Self::oneshot_send(tx, (), "")
                let _ = self.daemon_event_tx.send(DaemonEvent::ShutDown);
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
