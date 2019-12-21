use std::sync::mpsc;
use std::thread;

use futures::sync::oneshot;

use dnet_types::response::Response;
use dnet_types::settings::RunMode;
use tinc_plugin::TincTools;

use dnet_types::status::TunnelState;
use crate::cmd_api::management_server::ManagementCommand;
use crate::daemon_event_handle;
use crate::rpc::rpc_cmd::{RpcEvent, RpcProxyCmd};
use crate::daemon::{DaemonEvent, TunnelCommand};
use crate::info::{get_info, get_mut_info};
use crate::settings::get_settings;

pub struct DaemonEventMonitor {
    rpc_command_tx:         mpsc::Sender<RpcEvent>,
    daemon_event_tx:        mpsc::Sender<DaemonEvent>,
    tunnel_command_tx:      mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>,
}
impl DaemonEventMonitor {
    pub fn start(
        rpc_command_tx: mpsc::Sender<RpcEvent>,
        daemon_event_tx: mpsc::Sender<DaemonEvent>,
        tunnel_command_tx: mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>,
    ) -> mpsc::Sender<ManagementCommand> {
        let (daemon_monitor_cmd_tx, daemon_monitor_cmd_rx) = mpsc::channel();
        let daemon_event_monitor = DaemonEventMonitor {
            rpc_command_tx,
            daemon_event_tx,
            tunnel_command_tx,
        };
        thread::spawn(||daemon_event_monitor.spawn(daemon_monitor_cmd_rx));
        daemon_monitor_cmd_tx
    }

    fn spawn(mut self, daemon_monitor_cmd_rx: mpsc::Receiver<ManagementCommand>) {
        while let Ok(cmd) = daemon_monitor_cmd_rx.recv() {
            match cmd {
                ManagementCommand::Connect(tx) => {
                    let rpc_command_tx = self.rpc_command_tx.clone();
                    daemon_event_handle::connect::connect(
                        tx,
                        rpc_command_tx,
                    );
                }

                ManagementCommand::TeamDisconnect(tx, team_id) => {
                    let rpc_command_tx = self.rpc_command_tx.clone();
                    daemon_event_handle::disconnect_team::disconnect_team(
                        tx,
                        team_id,
                        rpc_command_tx);
                }

                ManagementCommand::Status(ipc_tx) => {
                    let info =  get_info().lock().unwrap();
                    let status = info.status.clone();
                    let vip = info.tinc_info.vip.clone();
                    let data = serde_json::json!({
                    "status": status,
                    "vip": vip,
                });
                    let response = Response::success().set_data(Some(data));
                    let _ = Self::oneshot_send(ipc_tx, response, "");
                }

                ManagementCommand::GroupInfo(ipc_tx, team_id) => {
                    daemon_event_handle::group_info::handle_group_info(
                        ipc_tx,
                        Some(team_id),
                    );
                }

                ManagementCommand::GroupUsers(ipc_tx, team_id) => {
                    let rpc_command_tx = self.rpc_command_tx.clone();
                    daemon_event_handle::group_users::handle_group_users(
                        ipc_tx,
                        rpc_command_tx,
                        team_id,
                    );
                }

                ManagementCommand::GroupList(ipc_tx) => {
                    daemon_event_handle::group_info::handle_group_info(
                        ipc_tx,
                        None,
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
                    daemon_event_handle::login::handle_login(
                        ipc_tx, user, rpc_command_tx);
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
                                if !host.contains("proxy") {
                                    send_to_rpc = true;
                                }
                            }
                        }
                        dnet_types::tinc_host_status_change::HostStatusChange::HostDown(host) => {
                            if let Some(vip) = TincTools::get_vip_by_filename(host) {
                                get_mut_info().lock().unwrap().tinc_info.remove_current_connect(&vip);
                                if !host.contains("proxy") {
                                    send_to_rpc = true;
                                }
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
    }

    fn handle_group_join(&self,
                         ipc_tx:        oneshot::Sender<Response>,
                         team_id:       String,
    ) {
        let rpc_command_tx = self.rpc_command_tx.clone();
        thread::spawn( ||
            daemon_event_handle::group_join::group_join(
                ipc_tx,
                team_id,
                rpc_command_tx,
            )
        );
    }

    fn handle_group_out(&self,
                        ipc_tx:        oneshot::Sender<Response>,
                        team_id:       String,
    ) {
        let rpc_command_tx = self.rpc_command_tx.clone();
        let tunnel_command_tx = self.tunnel_command_tx.clone();
        thread::spawn( ||
            daemon_event_handle::group_out::group_out(
                ipc_tx,
                team_id,
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


    fn handle_tunnel_connected(&mut self) {
//        if let Err(e) = TincOperator::new().set_routing() {
//            error!("host_status_change tinc-up {:?}", e);
//        }
        get_mut_info().lock().unwrap().status.tunnel = TunnelState::Connected;

        let _ = self.rpc_command_tx.send(RpcEvent::TunnelConnected);
        let (res_tx, _res_rx) = mpsc::channel::<Response>();
        let _ = self.tunnel_command_tx.send((TunnelCommand::Connected, res_tx));
    }

    fn handle_tunnel_disconnected(&mut self) {
        get_mut_info().lock().unwrap().status.tunnel = TunnelState::Disconnected;
    }


    pub fn oneshot_send<T>(ipc_tx: oneshot::Sender<T>, t: T, msg: &'static str) {
        if ipc_tx.send(t).is_err() {
            warn!("Unable to send {} to management interface client", msg);
        }
    }
}