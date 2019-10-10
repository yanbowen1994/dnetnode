use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use dnet_types::response::Response;

use crate::daemon::DaemonEvent;
use crate::traits::RpcTrait;
use crate::rpc::rpc_cmd::{RpcCmd, RpcClientCmd};
use crate::settings::default_settings::HEARTBEAT_FREQUENCY_SEC;
use super::RpcClient;
use super::rpc_client;
use super::rpc_client::select_proxy;
use crate::info::get_mut_info;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Connection with conductor timeout")]
    RpcTimeout,

    #[error(display = "Connection with conductor timeout")]
    TeamNotFound,
}

#[derive(Eq, PartialEq)]
enum RunStatus {
    NotSendHearbeat,
    SendHearbeat,
//    Restart,
}

pub struct RpcMonitor {
    client:                     RpcClient,
    daemon_event_tx:            mpsc::Sender<DaemonEvent>,
    rpc_cmd_rx:                 mpsc::Receiver<RpcCmd>,
    run_status:                 RunStatus,
}

impl RpcTrait for RpcMonitor {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> mpsc::Sender<RpcCmd> {
        let (rpc_cmd_tx, rpc_cmd_rx) = mpsc::channel();

        let client = RpcClient::new();
        RpcMonitor {
            client,
            daemon_event_tx,
            rpc_cmd_rx,
            run_status: RunStatus::NotSendHearbeat,
        }.start_monitor();
        return rpc_cmd_tx;
    }
}

impl RpcMonitor {
    fn start_monitor(self) {
        // TODO async
        thread::spawn(|| self.run());
    }

    fn run(mut self) {
        let timeout_millis: u32 = HEARTBEAT_FREQUENCY_SEC * 1000;
        loop {
            self.run_status = RunStatus::NotSendHearbeat;
            self.init();
            loop {
                if let Ok(cmd) = self.rpc_cmd_rx.try_recv() {
                    if self.handle_rpc_cmd(cmd) {
                        break
                    }
                }

                if self.run_status == RunStatus::SendHearbeat {
                    let start = Instant::now();

                    if let Err(_) = self.exec_heartbeat() {
                        break
                    }

                    if let Err(_) = self.exec_online_proxy() {
                        break
                    }

                    if let Some(remaining) = Duration::from_millis(
                        timeout_millis.into())
                        .checked_sub(start.elapsed()) {
                        thread::sleep(remaining);
                    }
                }
            }
            // break -> init()
        }
    }

    // If return false restart rpc connect.
    fn handle_rpc_cmd(&mut self, cmd: RpcCmd) -> bool {
        match cmd {
            RpcCmd::Client(cmd) => {
                match cmd {
                    RpcClientCmd::StartHeartbeat => {
                        self.run_status = RunStatus::SendHearbeat;
                    },

                    RpcClientCmd::RestartRpcConnect => {
                        return false;
                    }

                    #[cfg(target_arc = "test")]
                    RpcClientCmd::JoinTeam(team_id, res_tx) => {
                        let response = self.handle_join_team(team_id);
                        let _ = res_tx.send(response);
                    }

                    #[cfg(target_arc = "test")]
                    RpcClientCmd::ReportDeviceSelectProxy(response_tx) => {
                        let response = self.handle_select_proxy();
                        let _ = response_tx.send(response);
                    }

                    _ => ()
                }
            }
            _ => ()
        }
        true
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
                return Err(Error::RpcTimeout);
            }
            thread::sleep(Duration::from_millis(1000));
        }
    }

    fn exec_online_proxy(&self) -> Result<()> {
        // get_online_proxy is not most important. If failed still return Ok.
        loop {
            let start = Instant::now();

            if let Ok(connect_to_vec) = self.client.client_get_online_proxy() {
                if let Ok(()) = select_proxy(connect_to_vec) {
                    return Ok(());
                }
            } else {
                error!("Get online proxy failed.");
            }

            if Instant::now().duration_since(start) > Duration::from_secs(5) {
                return Err(Error::RpcTimeout);
            }
            thread::sleep(Duration::from_millis(1000));
        }
    }
}

#[cfg(target_arc = "test")]
impl RpcMonitor {
    fn init(&self) {
        let _ = self.daemon_event_tx.send(DaemonEvent::RpcConnecting);

        // 初始化上报操作
        loop {
            info!("client_login");
            {
                if let Err(e) = self.client.client_login() {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            info!("client_key_report");
            {
                if let Err(e) = self.client.client_key_report() {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            info!("binding device");
            {
                if let Err(e) = self.client.binding_device() {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            info!("search_user_team");
            {
                if let Err(e) = self.client.search_user_team() {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }
            break
        }
        let _ = self.daemon_event_tx.send(DaemonEvent::RpcConnected);
    }

    fn handle_join_team(&self, team_id: String) -> Response {
        info!("handle_join_team");
        let response;
        if let Err(error) = self.client.join_team(team_id) {
            response = Response::internal_error().set_msg(error.to_string())
        } else {
            self.client.search_user_team()?;
            self.start_team(team_id)?;
            response = Response::success();
        }
        response
    }

    fn start_team(&self, team_id: String) -> Result<()> {
        let mut info = get_mut_info().lock().unwrap();
        for team in &info.teams {
            if team.team_id == team_id {
                info.client_info.running_teams.push(team.clone());
                return Ok(());
            }
        }
        return Err(Error::TeamNotFound);
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

//#[cfg(target_arc = "arm")]
impl RpcMonitor {
    fn init(&self) {
        let _ = self.daemon_event_tx.send(DaemonEvent::RpcConnecting);

        // 初始化上报操作
        loop {
            info!("client_login");
            {
                if let Err(e) = self.client.client_login() {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            info!("client_key_report");
            {
                if let Err(e) = self.client.client_key_report() {
                    error!("{:?}\n{}", e, e);
                    thread::sleep(std::time::Duration::from_secs(1));
                    continue
                }
            }

            info!("search_team_by_mac");
            {
                match self.client.search_team_by_mac() {
                    Ok(_) => (),
                    Err(rpc_client::Error::client_not_bound) => {
                        thread::sleep(std::time::Duration::from_secs(10));
                        continue
                    },
                    Err(e) => {
                        error!("{:?}\n{}", e, e);
                        thread::sleep(std::time::Duration::from_secs(1));
                        continue
                    },
                }
            }

            self.start_team();

            info!("client_get_online_proxy");
            {
                match self.client.client_get_online_proxy()
                    .and_then(|connect_to_vec|select_proxy(connect_to_vec)) {
                    Ok(_) => (),
                    Err(e) => {
                        error!("{:?}\n{}", e, e);
                        thread::sleep(std::time::Duration::from_secs(1));
                        continue
                    },
                }
            }

            break
        }
        let _ = self.daemon_event_tx.send(DaemonEvent::RpcConnected);
    }

    // init means copy info.team to info.client.running_teams
    // use for client run as muti-team.
    fn start_team(&self) {
        let mut info = get_mut_info().lock().unwrap();
        info.client_info.running_teams = info.teams.clone();
    }
}