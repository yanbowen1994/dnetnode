use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use dnet_types::response::Response;

use crate::daemon::{DaemonEvent, TunnelCommand};
use crate::traits::RpcTrait;
use crate::rpc::rpc_cmd::{RpcEvent, RpcClientCmd, ExecutorEvent};
use crate::settings::default_settings::HEARTBEAT_FREQUENCY_SEC;
use crate::info::get_mut_info;
use super::RpcClient;
use super::rpc_client;
use super::error::{Error as ClientError, Result};
use crate::rpc::Error;

#[derive(Eq, PartialEq)]
enum RunStatus {
    Login,
    SendHearbeat,
    Stop,
}

pub struct RpcMonitor {
    client:                     RpcClient,
    daemon_event_tx:            mpsc::Sender<DaemonEvent>,
    rpc_rx:                     mpsc::Receiver<RpcEvent>,
    rpc_tx:                     mpsc::Sender<RpcEvent>,
    run_status:                 RunStatus,
    executor_tx:                Option<mpsc::Sender<(ExecutorCmd, Option<mpsc::Sender<bool>>)>>,
}

impl RpcTrait for RpcMonitor {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> mpsc::Sender<RpcEvent> {
        let (rpc_tx, rpc_rx) = mpsc::channel();

        let client = RpcClient::new();
        RpcMonitor {
            client,
            daemon_event_tx,
            rpc_rx,
            rpc_tx: rpc_tx.clone(),
            run_status: RunStatus::Stop,
            executor_tx: None,
        }.start_monitor();
        rpc_tx
    }
}

impl RpcMonitor {
    // If return false restart rpc connect.
    fn start_cmd_recv(mut self) {
        let mut rpc_restart_tx_cache: Option<mpsc::Sender<Response>> = None;
        while let Ok(rpc_event) = self.rpc_rx.recv() {
            info!("rpc_event {:?}", rpc_event);
            match rpc_event {
                RpcEvent::TunnelConnected => {
                    self.run_status = RunStatus::SendHearbeat;
                    if let Some(executor_tx) = &self.executor_tx {
                        let _ = executor_tx.send((ExecutorCmd::Heartbeat, None));
                    }
                }

                RpcEvent::TunnelDisConnected => {
                    if self.run_status == RunStatus::SendHearbeat {
                        self.run_status = RunStatus::Login;
                    }
                    if let Some(executor_tx) = &self.executor_tx {
                        let _ = executor_tx.send((ExecutorCmd::HeartbeatStop, None));
                    }
                }

                RpcEvent::Client(cmd) => {
                    match cmd {
                        RpcClientCmd::FreshTeam(res_tx) => {
                            let response = self.handle_fresh_team();
                            let _ = res_tx.send(response);
                        }

                        RpcClientCmd::TeamUsers(team_id, res_tx) => {
                            let response = self.handle_get_users_by_team(team_id);
                            let _ = res_tx.send(response);
                        }

                        RpcClientCmd::Stop(res_tx) => {
                            self.stop_executor();
                            let _ = res_tx.send(true);
                        }

                        RpcClientCmd::RestartRpcConnect(rpc_restart_tx) => {
                            self.restart_executor();
                            rpc_restart_tx_cache = Some(rpc_restart_tx);
                        },

                        RpcClientCmd::ReportDeviceSelectProxy(response_tx) => {
                            let response = self.handle_select_proxy();
                            let _ = response_tx.send(response);
                        },

                        #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
                        RpcClientCmd::JoinTeam(team_id, res_tx) => {
                            let response = self.handle_connect_team(team_id);
                            let _ = res_tx.send(response);
                        },

                        #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
                        RpcClientCmd::OutTeam(team_id, res_tx) => {
                            let response = self.handle_out_team(team_id);
                            let _ = res_tx.send(response);
                        },
                        #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
                        _ => ()
                    }
                }
                RpcEvent::Executor(event) => {
                    match event {
                        ExecutorEvent::InitFinish => {
                            if let Some(rpc_restart_tx) = &rpc_restart_tx_cache {
                                let _ = rpc_restart_tx.send(Response::success());
                            }
                            let _ = self.daemon_event_tx.send(DaemonEvent::RpcConnected);
                        },
                        ExecutorEvent::InitFailed(response) => {
                            if let Some(rpc_restart_tx) = &rpc_restart_tx_cache {
                                let _ = rpc_restart_tx.send(response);
                            }
                            rpc_restart_tx_cache = None;
                        },
                        ExecutorEvent::NeedRestartTunnel => {
                            if let Err(e) = self.daemon_event_tx.send(
                                DaemonEvent::DaemonInnerCmd(TunnelCommand::Reconnect))
                            {
                                error!("self.daemon_event_tx.send(\
                                DaemonEvent::DaemonInnerCmd(TunnelCommand::Reconnect)) {:?}", e)
                            }
                        }
                    }
                }
                _ => ()
            }
        }
    }

    fn restart_executor(&mut self) {
        if let Some(tx) = &self.executor_tx {
            let (stop_tx, stop_rx) = mpsc::channel();
            if let Ok(_) = tx.send((ExecutorCmd::Stop, Some(stop_tx))) {
                let _ = stop_rx.recv();
            }
        }
        self.start_executor()
    }

    fn start_executor(&mut self) {
        let executor = Executor::new(self.rpc_tx.clone());
        let executor_tx = executor.executor_tx.clone();
        executor.spawn();
        self.executor_tx= Some(executor_tx);
    }

    fn stop_executor(&mut self) {
        if let Some(tx) = &self.executor_tx {
            let (stop_tx, stop_rx) = mpsc::channel();
            if let Ok(_) = tx.send((ExecutorCmd::Stop, Some(stop_tx))) {
                let _ = stop_rx.recv();
                self.executor_tx = None;
            }
        }
    }


    fn handle_get_users_by_team(&self, team_id: String) -> Response {
        info!("handle_get_users_by_team");
        match self.client.get_users_by_team(&team_id) {
            Ok(res) => {
                let data: Vec<serde_json::Value> = res.iter()
                    .map(|user| {
                        user.to_json()
                    })
                    .collect();
                let data = serde_json::Value::Array(data);
                Response::success().set_data(Some(data))
            },
            Err(Error::http(code)) => Response::new_from_code(code),
            Err(e) => Response::internal_error().set_msg(e.to_string()),
        }
    }

    fn handle_select_proxy(&self) -> Response {
        info!("handle_select_proxy");
        let response;
        match self.client.device_select_proxy() {
            Ok(_) => response = Response::success(),
            Err(e) => response = e.to_response(),
        }
        response
    }
}

#[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
impl RpcMonitor {
    fn start_monitor(self) {
        thread::spawn(|| self.start_cmd_recv());
    }

    fn handle_connect_team(&self, team_id: String) -> Response {
        info!("handle_connect_team");
        if let Err(error) = self.client.join_team(&team_id) {
            let response = match error {
                Error::http(code) => Response::new_from_code(code),
                _ => Response::internal_error().set_msg(error.to_string()),
            };
            return response;
        } else {
            info!("search_user_team");
            if let Err(error) = self.client.search_user_team() {
                let response = match error {
                    Error::http(code) => Response::new_from_code(code),
                    _ => Response::internal_error().set_msg(error.to_string()),
                };
                return response;
            }
        }
        info!("handle_connect_team success");
        Response::success()
    }

    fn handle_out_team(&self, team_id: String) -> Response {
        info!("handle_out_team");
        if let Err(error) = self.client.out_team(&team_id) {
            let response = match error {
                Error::http(code) => Response::new_from_code(code),
                _ => Response::internal_error().set_msg(error.to_string()),
            };
            return response;
        } else {
            if let Err(error) = self.client.search_user_team() {
                return Response::internal_error().set_msg(error.to_string());
            }
        }
        Response::success()
    }

    fn handle_fresh_team(&self) -> Response {
        if let Err(error) = self.client.search_user_team() {
            let res = error.to_response();
            return res;
        }
        else {
            return Response::success();
        }
    }
}

#[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
impl RpcMonitor {
    fn start_monitor(mut self) {
        self.start_executor();
        thread::spawn(|| self.start_cmd_recv());
    }

    // init means copy info.team to info.client.running_teams
    // use for client run as muti-team.
//    fn start_team(&self) {
//        let mut info = get_mut_info().lock().unwrap();
//        info.teams.running_teams = info.teams.running_teams.clone();
//    }

    fn handle_fresh_team(&self) -> Response {
        if let Err(error) = self.client.search_team_by_mac() {
            return error.to_response();
        }
        else {
            return Response::success();
        }
    }
}

enum ExecutorCmd {
    Stop,
    Heartbeat,
    HeartbeatStop,
}

struct Executor {
    client:             RpcClient,
    executor_rx:        mpsc::Receiver<(ExecutorCmd, Option<mpsc::Sender<bool>>)>,
    executor_tx:        mpsc::Sender<(ExecutorCmd, Option<mpsc::Sender<bool>>)>,
    rpc_tx:             mpsc::Sender<RpcEvent>,
}

impl Executor {
    fn new(rpc_tx: mpsc::Sender<RpcEvent>) -> Self {
        let (executor_tx, executor_rx) = mpsc::channel();
        Self {
            client: RpcClient{},
            executor_rx,
            executor_tx,
            rpc_tx,
        }
    }

    fn spawn(self) {
        thread::spawn(||self.start_monitor());
    }

    fn start_monitor(self) {
        let timeout_millis: u32 = 1000;
        let mut init_success = false;
        let mut send_heartbeat = false;
        let mut heartbeat_start = Instant::now() - Duration::from_secs(20);
        let mut fresh_team = Instant::now() - Duration::from_secs(20);
        #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
            let route_not_bound_sleep = Instant::now();
        loop {
            let start = Instant::now();

            if let Some(remaining) = Duration::from_millis(3000)
                .checked_sub(start.elapsed()) {
                thread::sleep(remaining);
            }

            if let Ok((cmd, tx)) = self.executor_rx.try_recv() {
                match cmd {
                    ExecutorCmd::Stop => {
                        if let Some(tx) = tx {
                            let _ = tx.send(true);
                        }
                        return;
                    },
                    ExecutorCmd::Heartbeat => {
                        send_heartbeat = true;
                    },
                    ExecutorCmd::HeartbeatStop => {
                        send_heartbeat = false;
                    },
                }
            }
            if init_success && send_heartbeat {
                if start - heartbeat_start > Duration::from_secs(HEARTBEAT_FREQUENCY_SEC as u64) {
                    if let Ok(_) = self.exec_online_proxy() {
                        heartbeat_start = start;
                        info!("Rpc Executor get online proxy.");
                    } else {
                        error!("exec_online_proxy failed.");
                        init_success = false;
                    }
                }

                if start - fresh_team > Duration::from_secs(5) {
                    fresh_team = start;
                    let _ = self.client.search_team_by_mac();
                }
            }

            if !init_success {
                match self.init() {
                    Ok(_) => init_success = true,
                    Err(e) => {
                        let (need_return, res) = match &e {
                            Error::http(code) => {
                                let res = Response::new_from_code(*code);
                                (true, res)
                            },
                            _ => {
                                error!("rpc init unknown error {:?}", e);
                                let res = Response::internal_error();
                                (true, res)
                            },
                        };

                        if let Err(send_err) = self.rpc_tx.send(
                            RpcEvent::Executor(ExecutorEvent::InitFailed(res))) {
                            error!("self.rpc_tx.send(\
                            RpcEvent::Executor(ExecutorEvent::InitFailed(e))) {:?}", send_err);
                            return;
                        }
                        if need_return {
                            return;
                        }
                    }
                }

//                #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
//                    {
//                        if Instant::now() - route_not_bound_sleep > Duration::from_secs(10) {
//                            match self.init() {
//                                Ok(_) => init_success = true,
//                                Err(RpcError::client_not_bound) => route_not_bound_sleep = Instant::now(),
//                                _ => (),
//                            }
//                        }
//                    }
                if let Err(_) = self.rpc_tx.send(RpcEvent::Executor(ExecutorEvent::InitFinish)) {
                    return;
                }
                if init_success {
                    info!("rpc init success");
                }
            }

            if let Some(remaining) = Duration::from_millis(
                timeout_millis.into())
                .checked_sub(start.elapsed()) {
                thread::sleep(remaining);
            }
        }
    }

    fn exec_online_proxy(&self) -> Result<()> {
//         get_online_proxy is not most important. If failed still return Ok.
        loop {
            let start = Instant::now();
            match self.client.client_get_online_proxy() {
                Ok(connect_to_vec) => {
                    if let Ok(tunnel_restart) = rpc_client::select_proxy(connect_to_vec) {
                        if tunnel_restart {
                            let _ = self.rpc_tx.send(RpcEvent::Executor(ExecutorEvent::NeedRestartTunnel));
                        }
                        return Ok(());
                    }
                }
                Err(e) => error!("{:?}", e.to_response())
            }

            if Instant::now().duration_since(start) > Duration::from_secs(5) {
                return Err(ClientError::RpcTimeout);
            }
            thread::sleep(Duration::from_millis(1000));
        }
    }

    #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
    fn init(&self) -> std::result::Result<(), Error> {
        info!("client_login");
        self.client.client_login()?;
        info!("device_add");
        self.client.device_add()?;
        info!("search_user_team");
        self.client.search_user_team()?;
        info!("client_get_online_proxy");
        let connect_to = self.client.client_get_online_proxy()?;
        let mut info = get_mut_info().lock().unwrap();
        info.tinc_info.connect_to = connect_to;
        Ok(())
    }

    #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
    fn init(&self) -> std::result::Result<(), Error> {
        info!("client_login");
        self.client.client_login()?;
        info!("device_add");
        self.client.device_add()?;
        info!("client_get_online_proxy");
        let connect_to = self.client.client_get_online_proxy()?;
        let mut info = get_mut_info().lock().unwrap();
        info.tinc_info.connect_to = connect_to;
        Ok(())
    }
}