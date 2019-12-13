use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use dnet_types::response::Response;

use crate::daemon::{DaemonEvent, TunnelCommand};
use crate::traits::RpcTrait;
use crate::rpc::rpc_cmd::{RpcEvent, RpcClientCmd, ExecutorEvent};
use crate::settings::default_settings::HEARTBEAT_FREQUENCY_SEC;
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
                        let _ = executor_tx.send((ExecutorCmd::Start, None));
                    }
                }

                RpcEvent::TunnelDisConnected => {
                    if self.run_status == RunStatus::SendHearbeat {
                        self.run_status = RunStatus::Login;
                    }
                    if let Some(executor_tx) = &self.executor_tx {
                        let _ = executor_tx.send((ExecutorCmd::Stop, None));
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
                        #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
                        RpcClientCmd::DisconnectTeam(team_id, res_tx) => {
                                let response = self.handle_disconnect_team(team_id);
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
                            else {
                                if response.code != 623 {
                                    error!("ExecutorEvent::InitFailed {:?}", response);
                                }
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
            Err(e) => e.to_response(),
        }
    }

    fn handle_select_proxy(&self) -> Response {
        info!("handle_select_proxy");
        match self.client.client_get_online_proxy() {
            Ok(connect_to_vec) => {
                if let Ok(tunnel_restart) = rpc_client::select_proxy(connect_to_vec) {
                    if tunnel_restart {
                        let _ = self.rpc_tx.send(RpcEvent::Executor(ExecutorEvent::NeedRestartTunnel));
                    }
                }
            }
            Err(e) => {
                error!("{:?}", e.to_response());
                return e.to_response();
            }
        }
        match self.client.device_select_proxy() {
            Ok(_) => return Response::success(),
            Err(e) => return e.to_response(),
        }
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
            info!("connect_team");
            if let Err(error) = self.client.connect_team(&team_id) {
                return error.to_response();
            }
        }
        info!("handle_connect_team success");
        Response::success()
    }

    fn handle_out_team(&self, team_id: String) -> Response {
        info!("handle_out_team");
        if let Err(error) = self.client.out_team(&team_id) {
            let response = error.to_response();
            error!("handle_out_team {:?}", response);
            return response;
        } else {
            if let Err(error) = self.client.disconnect_team(&team_id) {
                return Response::internal_error().set_msg(error.to_string());
            }
        }
        Response::success()
    }

    fn handle_disconnect_team(&self, team_id: String) -> Response {
        info!("handle_disconnect_team");
        if let Err(error) = self.client.disconnect_team(&team_id) {
            let response = error.to_response();
            error!("handle_disconnect_team {:?}", response);
            return response;
        }
        Response::success()
    }

    fn handle_fresh_team(&self) -> Response {
        if let Err(error) = self.client.search_team_by_user() {
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
    Start,
}

#[derive(PartialEq)]
enum ExecutorStatus {
    Inited,
    Uninit,
    Running,
}

struct Executor {
    client:             RpcClient,
    executor_rx:        mpsc::Receiver<(ExecutorCmd, Option<mpsc::Sender<bool>>)>,
    executor_tx:        mpsc::Sender<(ExecutorCmd, Option<mpsc::Sender<bool>>)>,
    rpc_tx:             mpsc::Sender<RpcEvent>,
    status:             ExecutorStatus,
}

impl Executor {
    fn new(rpc_tx: mpsc::Sender<RpcEvent>) -> Self {
        let (executor_tx, executor_rx) = mpsc::channel();
        Self {
            client: RpcClient{},
            executor_rx,
            executor_tx,
            rpc_tx,
            status:     ExecutorStatus::Uninit,
        }
    }

    fn spawn(self) {
        thread::spawn(||self.start_monitor());
    }

    fn start_monitor(mut self) {
        let timeout_millis: u32 = 1000;
        let mut heartbeat_start = Instant::now() - Duration::from_secs(20);
        let mut fresh_team = Instant::now() - Duration::from_secs(20);
        loop {
            if let Ok((cmd, tx))
            = self.executor_rx.recv_timeout(Duration::from_secs(3)) {
                match cmd {
                    ExecutorCmd::Stop => {
                        if let Some(tx) = tx {
                            let _ = tx.send(true);
                        }
                        return;
                    },
                    ExecutorCmd::Start => {
                        if self.status == ExecutorStatus::Inited {
                            self.status = ExecutorStatus::Running;
                            if let Some(tx) = tx {
                                let _ = tx.send(true);
                            }
                        }
                        else {
                            if let Some(tx) = tx {
                                let _ = tx.send(false);
                            }
                        }
                    },
                }
            }

            let start = Instant::now();

            if self.status == ExecutorStatus::Running {
                if start - heartbeat_start > Duration::from_secs(HEARTBEAT_FREQUENCY_SEC as u64) {
                    if let Ok(_) = self.exec_online_proxy() {
                        heartbeat_start = start;
                        info!("Rpc Executor get online proxy.");
                    } else {
                        error!("exec_online_proxy failed.");
                        self.status = ExecutorStatus::Uninit;
                    }
                }

                if start - fresh_team > Duration::from_secs(5) {
                    fresh_team = start;
                    let _ = self.client.search_team_by_mac();
                }
            }

            if self.status == ExecutorStatus::Uninit {
                match self.init() {
                    Ok(_) => {
                        if let Err(_) = self.rpc_tx.send(RpcEvent::Executor(ExecutorEvent::InitFinish)) {
                            return;
                        }
                        self.status = ExecutorStatus::Inited;
                        info!("rpc init success");
                    },
                    Err(e) => {
                        let res = e.to_response();
                        error!("rpc init {:?}", res);
                        if res.code == 411
                            || res.code == 405
                        {
                            if let Err(send_err) = self.rpc_tx.send(
                                RpcEvent::Executor(ExecutorEvent::InitFailed(res))) {
                                error!("self.rpc_tx.send(\
                                RpcEvent::Executor(ExecutorEvent::InitFailed(e))) {:?}", send_err);
                            }
                            return;
                        }
                    }
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
        info!("search_team_by_user");
        self.client.search_team_by_user()?;
        Ok(())
    }

    #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
    fn init(&self) -> std::result::Result<(), Error> {
        info!("client_login");
        self.client.client_login()?;
        info!("device_add");
        self.client.device_add()?;
        info!("client_get_online_proxy");
        Ok(())
    }
}