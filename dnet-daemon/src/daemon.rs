use std::sync::mpsc;
use std::thread;

use futures::sync::oneshot;

use dnet_types::states::{DaemonExecutionState, TunnelState, State, RpcState};

use crate::traits::TunnelTrait;
use crate::info::{self, Info, get_mut_info};
use crate::rpc::{self, RpcMonitor};
use crate::tinc_manager::{TincMonitor, TincOperator};
use crate::cmd_api::management_server::{ManagementInterfaceServer, ManagementCommand, ManagementInterfaceEventBroadcaster};
use crate::mpsc::IntoSender;
use crate::settings::get_settings;
use dnet_types::settings::RunMode;
use dnet_types::response::Response;
use crate::rpc::rpc_cmd::{RpcEvent, RpcProxyCmd};
use std::time::Duration;
use super::daemon_event_handle;
use tinc_plugin::TincTools;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Tinc can't supported ipv6")]
    UnsupportedTunnel,

    /// Error in the management interface
    #[error(display = "Unable to start management interface server")]
    StartManagementInterface(#[error(cause)] ipc_server::Error),

    #[error(display = "Unable to start management interface server")]
    InfoError(#[error(cause)] info::Error),

    #[error(display = "Tunnel init failed.")]
    TunnelInit(#[error(cause)] tinc_plugin::TincOperatorError),
}

#[derive(Clone, Debug)]
pub enum TunnelCommand {
    Connect,
    Connected,
    Disconnect,
    Disconnected,
    Reconnect,
}

pub enum DaemonEvent {
    // -> self.Status.rpc.Connected
    RpcConnected,

    // if init -> self.Status.rpc.Connecting
    // else -> self.Status.rpc.ReConnecting
    RpcConnecting,

    // ->
    TunnelInitFailed(String),

    DaemonInnerCmd(TunnelCommand),

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
    tunnel_command_tx:      mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>,
    rpc_command_tx:         mpsc::Sender<RpcEvent>,
}

impl Daemon {
    pub fn start() -> Result<Self> {
        #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
            {
                info!("start dnet firewall config.");
                let tunnel_port = get_settings().tinc.port;
                router_plugin::firewall::start_firewall(tunnel_port);
            }

        let (daemon_event_tx, daemon_event_rx) = mpsc::channel();

        let _ = crate::set_shutdown_signal_handler(daemon_event_tx.clone());

        let _event_broadcaster = Self::start_management_interface(daemon_event_tx.clone())?;

        TincOperator::new().init()
            .map_err(Error::TunnelInit)?;

        info!("Init local info.");
        Info::new().map_err(Error::InfoError)?;

        let run_mode = &get_settings().common.mode;
        let rpc_command_tx;
        if run_mode == &RunMode::Proxy || run_mode == &RunMode::Center {
            rpc_command_tx = RpcMonitor::new::<rpc::proxy::RpcMonitor>(daemon_event_tx.clone());
        }
        else {
            rpc_command_tx = RpcMonitor::new::<rpc::client::RpcMonitor>(daemon_event_tx.clone());
        }

        let (tinc, tunnel_command_tx) =
            TincMonitor::new(daemon_event_tx.clone());
        tinc.start_monitor();

        Ok(Daemon {
            daemon_event_tx,
            daemon_event_rx,
            status: State::new(),
            tunnel_command_tx,
            rpc_command_tx,
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
            DaemonEvent::TunnelInitFailed(err_str) => {
                self.status.tunnel = TunnelState::TunnelInitFailed(err_str);
            },
            DaemonEvent::DaemonInnerCmd(cmd) => {
                let (res_tx, res_rx) = mpsc::channel::<Response>();
                let _ = self.tunnel_command_tx.send((cmd.clone(), res_tx));
                let _ = res_rx.recv_timeout(Duration::from_secs(3))
                    .map(|res|{
                        if res.code != 200 {
                            error!("DaemonInnerCmd::{:?} exec failed. error: {:?}", cmd.clone(), res.msg);
                        }
                    })
                    .map_err(|_| {
                        error!("DaemonInnerCmd::{:?} exec failed. error: Respones recv timeout.", cmd.clone())
                    });
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
        let (res_tx, res_rx) = mpsc::channel::<Response>();
        let _ = self.tunnel_command_tx.send((TunnelCommand::Disconnect, res_tx));
        let _ = res_rx.recv_timeout(Duration::from_secs(5));
        #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
            {
                info!("stop dnet firewall config.");
                let tunnel_port = get_settings().tinc.port;
                router_plugin::firewall::stop_firewall(tunnel_port);
            }
        self.status.daemon = DaemonExecutionState::Finished;
    }

    fn handle_tunnel_connected(&mut self) {
//        if let Err(e) = TincOperator::new().set_routing() {
//            error!("host_status_change tinc-up {:?}", e);
//        }
        self.status.tunnel = TunnelState::Connected;

        let _ = self.rpc_command_tx.send(RpcEvent::TunnelConnected);
        let (res_tx, _res_rx) = mpsc::channel::<Response>();
        let _ = self.tunnel_command_tx.send((TunnelCommand::Connected, res_tx));
    }

    fn handle_tunnel_disconnected(&mut self) {
        self.status.tunnel = TunnelState::Disconnected;
    }

    fn handle_ipc_command_event(&mut self, cmd: ManagementCommand) {
        match cmd {
            ManagementCommand::Connect(tx) => {
                let status = self.status.clone();
                let tunnel_command_tx = self.tunnel_command_tx.clone();
                let rpc_command_tx = self.rpc_command_tx.clone();
                thread::spawn(|| daemon_event_handle::connect::connect(
                    tx,
                    status,
                    rpc_command_tx,
                    tunnel_command_tx)
                );
            }

            ManagementCommand::TeamDisconnect(tx, team_id) => {
                let status = self.status.clone();
                let rpc_command_tx = self.rpc_command_tx.clone();
                thread::spawn(move|| daemon_event_handle::disconnect_team::disconnect_team(
                    tx,
                    status,
                    team_id,
                    rpc_command_tx)
                );
            }

            ManagementCommand::State(ipc_tx) => {
//                let mut response = CommandResponse::success();
//                response.data = Some(to_value(&self.status.rpc).unwrap());
                let state = self.status.clone();
                let _ = Self::oneshot_send(ipc_tx, state, "");
            }

            ManagementCommand::GroupInfo(ipc_tx, team_id) => {
                let rpc_command_tx = self.rpc_command_tx.clone();
                let status = self.status.clone();
                thread::spawn(|| daemon_event_handle::group_info::handle_group_info(
                    ipc_tx,
                    rpc_command_tx,
                    Some(team_id),
                    status),
                );
            }

            ManagementCommand::GroupUsers(ipc_tx, team_id) => {
                let rpc_command_tx = self.rpc_command_tx.clone();
                thread::spawn(|| daemon_event_handle::group_users::handle_group_users(
                    ipc_tx,
                    rpc_command_tx,
                    team_id)
                );
            }

            ManagementCommand::GroupList(ipc_tx) => {
                let rpc_command_tx = self.rpc_command_tx.clone();
                let status = self.status.clone();
                thread::spawn(|| daemon_event_handle::group_info::handle_group_info(
                    ipc_tx,
                    rpc_command_tx,
                    None,
                    status)
                );
            }

            ManagementCommand::GroupJoin(ipc_tx, team_id) => {
                self.handle_group_join(ipc_tx, team_id);
            }

            ManagementCommand::GroupOut(ipc_tx, team_id) => {
                self.handle_group_out(ipc_tx, team_id);
            }

            ManagementCommand::Login(ipc_tx, user) => {
                let rpc_command_tx = self.rpc_command_tx.clone();
                thread::spawn(move ||
                    daemon_event_handle::login::handle_login(
                        ipc_tx, user, rpc_command_tx));
            }

            ManagementCommand::Logout(ipc_tx) => {
                self.handle_logout(ipc_tx);
            }

            ManagementCommand::HostStatusChange(ipc_tx, host_status_change) => {
                // No call back.
                let _ = Self::oneshot_send(ipc_tx, (), "");
                let mut send_to_rpc = false;
                // TODO tunnel ipc -> monitor
                match &host_status_change {
                    dnet_types::tinc_host_status_change::HostStatusChange::TincUp => {
                        self.handle_tunnel_connected()
                    },
                    dnet_types::tinc_host_status_change::HostStatusChange::TincDown => {
                        self.handle_tunnel_disconnected()
                    },
                    dnet_types::tinc_host_status_change::HostStatusChange::HostUp(host) => {
                        if let Some(vip) = TincTools::get_vip_by_filename(host) {
                            get_mut_info().lock().unwrap().tinc_info.current_connect.push(vip);
                            send_to_rpc = true;
                        }
                    }
                    dnet_types::tinc_host_status_change::HostStatusChange::HostDown(host) => {
                        if let Some(vip) = TincTools::get_vip_by_filename(host) {
                            get_mut_info().lock().unwrap().tinc_info.remove_current_connect(&vip);
                            send_to_rpc = true;
                        }
                    }
                }
                if send_to_rpc {
                    let run_mode = &get_settings().common.mode;
                    if *run_mode == RunMode::Proxy ||
                        *run_mode == RunMode::Center {
                        if let Err(e) = self.rpc_command_tx.send(
                            RpcEvent::Proxy(
                                RpcProxyCmd::HostStatusChange(host_status_change)
                            )
                        ) {
                            error!("{:?}", e);
                        };
                    }
                }
            }

            ManagementCommand::Shutdown(ipc_tx) => {
                let _ = self.daemon_event_tx.send(DaemonEvent::ShutDown);

                let command_response = Response::success();

                info!("Shutdown by cli command.");

                let _ = Self::oneshot_send(ipc_tx, command_response, "");
            }
        }
    }

    fn handle_rpc_connected(&mut self) {
        #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
            {
                self.status.tunnel = TunnelState::Connecting;
                let (res_tx, _) = mpsc::channel();
                let _ = self.tunnel_command_tx.send((TunnelCommand::Connect, res_tx));
            }
        #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
            {
                let mut tunnel_auto_connect = false;
                self.status.rpc = RpcState::Connected;
                let run_mode = get_settings().common.mode.clone();

                if run_mode == RunMode::Proxy || run_mode == RunMode::Center {
                    tunnel_auto_connect = true;
                }
                else {
                    let auto_connect = get_settings().client.auto_connect;
                    if auto_connect == true {
                        if self.status.tunnel == TunnelState::Disconnected ||
                            self.status.tunnel == TunnelState::Disconnecting {
                            tunnel_auto_connect = true;
                        }
                    }
                }

                if tunnel_auto_connect {
                    info!("tunnel auto connect");
                    let _response = daemon_event_handle::tunnel::send_tunnel_connect(
                        self.tunnel_command_tx.clone(),
                    );
                }
            }
    }

    pub fn oneshot_send<T>(ipc_tx: oneshot::Sender<T>, t: T, msg: &'static str) {
        if ipc_tx.send(t).is_err() {
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
        let path = dnet_path::ipc_path();
        let server =
            ManagementInterfaceServer::start(&path, event_tx).map_err(Error::StartManagementInterface)?;
        info!("Management interface listening on {}", server.socket_path());

        Ok(server)
    }

    fn spawn_management_interface_wait_thread(
        server: ManagementInterfaceServer,
        _exit_tx: mpsc::Sender<DaemonEvent>,
    ) {
        thread::spawn(move || {
            server.wait();
            info!("Management interface shut down");
//            let _ = exit_tx.send(DaemonEvent::ManagementInterfaceExited);
        });
    }

    fn handle_group_join(&self,
                         ipc_tx:        oneshot::Sender<Response>,
                         team_id:       String,
    ) {
        let status = self.status.clone();
        let rpc_command_tx = self.rpc_command_tx.clone();
        thread::spawn( ||
            daemon_event_handle::group_join::group_join(
                ipc_tx,
                team_id,
                status,
                rpc_command_tx,
            )
        );
    }

    fn handle_group_out(&self,
                         ipc_tx:        oneshot::Sender<Response>,
                         team_id:       String,
    ) {
        let status = self.status.clone();
        let rpc_command_tx = self.rpc_command_tx.clone();
        let tunnel_command_tx = self.tunnel_command_tx.clone();
        thread::spawn( ||
            daemon_event_handle::group_out::group_out(
                ipc_tx,
                team_id,
                status,
                rpc_command_tx,
                tunnel_command_tx,
            )
        );
    }

    fn handle_logout(&self,
                     ipc_tx:        oneshot::Sender<Response>,
    ) {
        let rpc_command_tx = self.rpc_command_tx.clone();
        let tunnel_command_tx = self.tunnel_command_tx.clone();
        thread::spawn(move ||
            daemon_event_handle::logout::handle_logout(
                ipc_tx,
                rpc_command_tx,
                tunnel_command_tx,
            )
        );
    }
}