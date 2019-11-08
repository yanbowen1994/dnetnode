use std::sync::mpsc;
use std::thread;

use futures::sync::oneshot;

use dnet_types::states::{DaemonExecutionState, TunnelState, State, RpcState};

use crate::traits::TunnelTrait;
use crate::info::{self, Info, get_info};
use crate::rpc::{self, RpcMonitor};
use crate::tinc_manager::{TincMonitor, TincOperator};
use crate::cmd_api::ipc_server::{ManagementInterfaceServer, ManagementCommand, ManagementInterfaceEventBroadcaster};
use crate::mpsc::IntoSender;
use crate::settings::{get_settings, get_mut_settings};
use dnet_types::settings::RunMode;
use dnet_types::response::Response;
use crate::rpc::rpc_cmd::{RpcEvent, RpcClientCmd, RpcProxyCmd};
use std::time::Duration;
use std::sync::mpsc::{channel, Sender};
use super::daemon_event_handle;

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
    Disconnect,
    Reconnect,
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
        #[cfg(any(target_arch = "arm", feature = "router_debug"))]
            {
                info!("start dnet firewall config.");
                router_plugin::firewall::start_firewall();
            }

        let (daemon_event_tx, daemon_event_rx) = mpsc::channel();

        let _ = crate::set_shutdown_signal_handler(daemon_event_tx.clone());

        let event_broadcaster = Self::start_management_interface(daemon_event_tx.clone())?;
//        event_broadcaster.

        TincOperator::new().init()
            .map_err(Error::TunnelInit)?;

        info!("Init local info.");
        Info::new().map_err(Error::InfoError)?;

        let run_mode = &get_settings().common.mode;
        let rpc_command_tx;
        if run_mode == &RunMode::Proxy {
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
            DaemonEvent::TunnelConnected => {
                self.handle_tunnel_connected();
            },
            DaemonEvent::TunnelDisconnected => {
                self.status.tunnel = TunnelState::Disconnected;
            },
            DaemonEvent::TunnelInitFailed(err_str) => {
                self.status.tunnel = TunnelState::TunnelInitFailed(err_str);
            },
            DaemonEvent::DaemonInnerCmd(cmd) =>  {
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
        let _ = res_rx.recv_timeout(Duration::from_secs(3));
        #[cfg(any(target_arch = "arm", feature = "router_debug"))]
            {
                info!("stop dnet firewall config.");
                router_plugin::firewall::stop_firewall();
            }
        self.status.daemon = DaemonExecutionState::Finished;
    }

    fn handle_tunnel_connected(&mut self) {
        self.status.tunnel = TunnelState::Connected;
        if get_settings().common.mode == RunMode::Client {
            let _ =self.rpc_command_tx.send(RpcEvent::Client(RpcClientCmd::HeartbeatStart));
        }
    }

    fn handle_ipc_command_event(&mut self, cmd: ManagementCommand) {
        match cmd {
            ManagementCommand::TunnelConnect(tx) => {
                let run_mode = get_settings().common.mode.clone();
                let res = if run_mode == RunMode::Proxy {
                    Response::internal_error().set_msg("Invalid command in proxy mode".to_owned())
                }
                else if self.status.tunnel == TunnelState::Connecting
                    || self.status.tunnel == TunnelState::Connected {
                    Response::internal_error().set_msg("Invalid command. Currently connected.".to_owned())
                }
                else {
//              TODO async
                    let (rpc_response_tx, rpc_response_rx) = mpsc::channel();
                    let _ = self.rpc_command_tx.send(RpcEvent::Client(RpcClientCmd::ReportDeviceSelectProxy(rpc_response_tx)));

                    if let Ok(rpc_res) = rpc_response_rx.recv() {
                        if rpc_res.code == 200 {
                            let tunnel_res = self.send_tunnel_connect();
                            tunnel_res
                        }
                        else {
                            Response::internal_error().set_msg(rpc_res.msg)
                        }
                    }
                    else {
                        Response::internal_error().set_msg("Exec failed.".to_owned())
                    }
                };
                let _ = Self::oneshot_send(tx, res, "");
            }

            ManagementCommand::TunnelDisconnect(tx) => {
                let (res_tx, res_rx) = mpsc::channel::<Response>();
                let _ = self.tunnel_command_tx.send((TunnelCommand::Disconnect, res_tx));
                let res = match res_rx.recv_timeout(Duration::from_secs(3)) {
                    Ok(res) => res,
                    Err(_) => Response::internal_error(),
                };

                let _ = Self::oneshot_send(tx, res, "");
            }

            ManagementCommand::State(tx) => {
//                let mut response = CommandResponse::success();
//                response.data = Some(to_value(&self.status.rpc).unwrap());
                let state = self.status.clone();
                let _ = Self::oneshot_send(tx, state, "");
            }

            ManagementCommand::GroupInfo(tx, team_id) => {
                let mut team = None;
                for team_info in &get_info().lock().unwrap().teams {
                    if team_info.team_id == team_id {
                        team = Some(team_info.clone());
                    }
                }
                let _ = Self::oneshot_send(tx, team, "");
            }

            ManagementCommand::GroupList(tx) => {
                let team =  get_info().lock().unwrap().teams.clone();
                let _ = Self::oneshot_send(tx, team, "");
            }

            ManagementCommand::GroupJoin(tx, team_id) => {
                let (res_tx, res_rx) = mpsc::channel();
                let _ = self.rpc_command_tx.send(RpcEvent::Client(RpcClientCmd::JoinTeam(team_id, res_tx)));
                thread::spawn(move || {
                    let response = match res_rx.recv_timeout(Duration::from_secs(3)) {
                        Ok(res) => res,
                        Err(_) => Response::exec_timeout(),
                    };
                    let _ = Self::oneshot_send(tx, response, "");
                });
            }

            ManagementCommand::Login(tx, user) => {
                let rpc_command_tx = self.rpc_command_tx.clone();
                thread::spawn(move ||daemon_event_handle::handle_login(tx, user, rpc_command_tx));
            }

            ManagementCommand::HostStatusChange(tx, host_status_change) => {
                // No call back.
                let _ = Self::oneshot_send(tx, (), "");

                // TODO tunnel ipc -> monitor
                match host_status_change {
                    dnet_types::tinc_host_status_change::HostStatusChange::TincUp => {
                        if let Err(e) = TincOperator::new().set_routing() {
                            error!("host_status_change tinc-up {:?}", e);
                        }
                    },
                    _ => (),
                }

                let _ = self.rpc_command_tx.send(
                    RpcEvent::Proxy(
                        RpcProxyCmd::HostStatusChange(host_status_change)
                    )
                );
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
        let mut tunnel_auto_connect = false;
        self.status.rpc = RpcState::Connected;
        let run_mode = get_settings().common.mode.clone();

        if run_mode == RunMode::Proxy {
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
            self.send_tunnel_connect();
        }
    }

    pub fn oneshot_send<T>(tx: oneshot::Sender<T>, t: T, msg: &'static str) {
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

    fn send_tunnel_connect(&self) -> Response {
        let (res_tx, res_rx) = mpsc::channel::<Response>();
        let _ = self.tunnel_command_tx.send((TunnelCommand::Connect, res_tx));
        let res = res_rx.recv_timeout(Duration::from_secs(3))
            .map(|res|{
                if res.code == 200 {
                    let _ = self.daemon_event_tx.send(DaemonEvent::TunnelConnected);
                }
                else {
                    error!("Tunnel connect failed. error: {:?}", res.msg);
                }
                res
            })
            .map_err(|_| {
                error!("Tunnel connect failed. error: Respones recv timeout.")
            })
            .unwrap_or(Response::exec_timeout());
        res
    }
}
