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
use super::rpc_client::{self, Error as SubError};
use super::error::{Error as ClientError, Result};

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
    fn start_monitor(self) {
        thread::spawn(|| self.start_cmd_recv());
    }

    // If return false restart rpc connect.
    fn start_cmd_recv(mut self) {
        let mut rpc_restart_tx_cache: Option<mpsc::Sender<Response>> = None;
        while let rpc_event = self.rpc_rx.recv().unwrap() {
            match rpc_event {
                RpcEvent::Client(cmd) => {
                    match cmd {
                        RpcClientCmd::HeartbeatStart => {
                            self.run_status = RunStatus::SendHearbeat;
                            if let Some(executor_tx) = &self.executor_tx {
                                let _ = executor_tx.send((ExecutorCmd::Heartbeat, None));
                            }
                        },

                        RpcClientCmd::Stop => {
                            self.stop_executor();
                        }

                        RpcClientCmd::RestartRpcConnect(rpc_restart_tx) => {
                            info!("restart rpc connect");
                            self.restart_executor();
                            rpc_restart_tx_cache = Some(rpc_restart_tx);
                        },

                        #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
                        RpcClientCmd::JoinTeam(team_id, res_tx) => {
                            let response = self.handle_join_team(team_id);
                            let _ = res_tx.send(response);
                        },

                        #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
                        RpcClientCmd::ReportDeviceSelectProxy(response_tx) => {
                            let response = self.handle_select_proxy();
                            let _ = response_tx.send(response);
                        },
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
                        ExecutorEvent::InitFailed(e) => {
                            let mut response = Response::internal_error();
                            response = response.set_msg(e.to_string());
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

    fn start_executor(&mut self)
    {
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
        #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
            let mut route_not_bound_sleep = Instant::now();
        loop {
            let start = Instant::now();
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
            if start - heartbeat_start > Duration::from_secs(HEARTBEAT_FREQUENCY_SEC as u64) {
                heartbeat_start = start;
                if send_heartbeat {
                    if init_success {
                        if let Err(_) = self.exec_heartbeat() {
                            init_success = false;
                        }
                    }

                    if init_success {
                        if let Err(_) = self.exec_online_proxy() {
                            init_success = false;
                        }
                    }
                }
                info!("Rpc Executor heartbeat.");
            }

            if !init_success {
                #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
                    {
                        match self.init() {
                            Ok(_) => init_success = true,
                            Err(e) => {
                                let need_return = match &e {
                                    SubError::Unauthorized => true,
                                    SubError::UserNotExist => true,
                                    _ => false,
                                };
                                if let Err(send_err) = self.rpc_tx.send(
                                    RpcEvent::Executor(ExecutorEvent::InitFailed(e))) {
                                    error!("self.rpc_tx.send(\
                                    RpcEvent::Executor(ExecutorEvent::InitFailed(e))) {:?}", send_err);
                                    return;
                                }
                                if need_return {
                                    return;
                                }
                            }
                        }
                    }

                #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
                    {
                        if Instant::now() - route_not_bound_sleep > Duration::from_secs(10) {
                            match self.init() {
                                Ok(_) => init_success = true,
                                Err(RpcError::client_not_bound) => route_not_bound_sleep = Instant::now(),
                                _ => (),
                            }
                        }
                    }
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

    // get_online_proxy with heartbeat (The client must get the proxy offline info in this way.)
    fn exec_heartbeat(&self) -> Result<()> {
        info!("client_heart_beat");
        loop {
            let start = Instant::now();
            if let Ok(_) = self.client.client_heartbeat() {
                return Ok(());
            } else {
                error!("Heart beat send failed.");
            }
            if Instant::now().duration_since(start) > Duration::from_secs(5) {
                return Err(ClientError::RpcTimeout);
            }
            thread::sleep(Duration::from_millis(1000));
        }
    }

    fn exec_online_proxy(&self) -> Result<()> {
//         get_online_proxy is not most important. If failed still return Ok.
//        info!("exec_online_proxy");
        loop {
            let start = Instant::now();
            if let Ok(connect_to_vec) = self.client.client_get_online_proxy() {
                if let Ok(tunnel_restart) = rpc_client::select_proxy(connect_to_vec) {
                    if tunnel_restart {
                        let _ = self.rpc_tx.send(RpcEvent::Executor(ExecutorEvent::NeedRestartTunnel));
                    }
                    return Ok(());
                }
            } else {
                error!("Get online proxy failed.");
            }

            if Instant::now().duration_since(start) > Duration::from_secs(5) {
                return Err(ClientError::RpcTimeout);
            }
            thread::sleep(Duration::from_millis(1000));
        }
    }

    #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
    fn init(&self) -> std::result::Result<(), SubError> {
        self.client.client_login()?;
        self.client.client_key_report()?;
        self.client.binding_device()?;
        self.client.search_user_team()?;
        Ok(())
    }

    #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
    fn init(&self) -> std::result::Result<(), SubError> {
        self.client.client_login()?;
        self.client.client_key_report()?;
        self.client.search_team_by_mac()?;
        self.start_team()?;
        self.client.client_get_online_proxy()?;
        self.client.connect_team_broadcast()?;
        Ok(())
    }
}

#[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
impl RpcMonitor {
    fn handle_join_team(&self, team_id: String) -> Response {
        info!("handle_join_team");
        if let Err(error) = self.client.join_team(&team_id) {
            return Response::internal_error().set_msg(error.to_string());
        } else {
            if let Err(error) = self.client.search_user_team() {
                return Response::internal_error().set_msg(error.to_string());
            }

            if let Err(error) = self.start_team(&team_id) {
                return Response::internal_error().set_msg(error.to_string());
            }
        }
        Response::success()
    }

    fn start_team(&self, team_id: &str) -> Result<()> {
        let mut info = get_mut_info().lock().unwrap();
        let mut add_running_team = vec![];

        let _ = info.teams
            .iter()
            .filter_map(|team| {
                if &team.team_id == team_id {
                    add_running_team.push(team.clone());
                    Some(())
                }
                else {
                    None
                }
            })
            .collect::<Vec<()>>();

        if add_running_team.len() > 0 {
            info.client_info.running_teams.append(&mut add_running_team);
            Ok(())
        }
        else {
            return Err(ClientError::TeamNotFound);
        }
    }

    fn handle_select_proxy(&self) -> Response {
        info!("handle_select_proxy");
        let response;
        match self.client.device_select_proxy() {
            Ok(_) => response = Response::success(),
            Err(e) => response = Response::internal_error().set_msg(e.to_string())
        }
        response
    }
}

#[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
impl RpcMonitor {
    // init means copy info.team to info.client.running_teams
    // use for client run as muti-team.
    fn start_team(&self) {
        let mut info = get_mut_info().lock().unwrap();
        info.client_info.running_teams = info.teams.clone();
    }
}