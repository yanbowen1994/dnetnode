use std::io;
use std::sync::mpsc;
use std::net::Shutdown;

use tinc_plugin::{PID_FILENAME, TincStream, TincTools};
use dnet_types::tinc_host_status_change::HostStatusChange;
use dnet_types::settings::RunMode;
use sandbox::route::{add_route, del_route};
use crate::settings::get_settings;
use crate::rpc::proxy::RpcClient;
use crate::settings::default_settings::TINC_INTERFACE;
use crate::daemon::DaemonEvent;

pub struct TincEventHandle {
    socket: Option<socket2::Socket>,
    tinc_pid: String,
    daemon_event_tx: mpsc::Sender<DaemonEvent>,
}

impl TincEventHandle {
    pub fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> Self {
        let tinc_pid = get_settings().common.home_path
            .join("tinc").join(PID_FILENAME)
            .to_str().unwrap().to_string();

        let tinc_event_handle = Self {
            socket:  None,
            tinc_pid,
            daemon_event_tx,
        };
        tinc_event_handle
    }

    pub fn recv(&mut self) -> Option<()> {
        if let None = self.socket {
            if self.subscribe() {
                self.connect_status_change(true);
            }
            else {
                return None;
            }
        }

        let socket = self.socket.as_ref().unwrap();
        match TincStream::recv_from_subscribe(socket) {
            Ok(res) => {
                if let Some(host_status_change) = self.recv_parse(&res) {
                    self.host_status_change_handle(host_status_change)
                }
                return Some(());
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::TimedOut {
                    return Some(());
                }
                else {
                    self.socket = None;
                    self.connect_status_change(false);
                    return None;
                }
            }
        }
    }

    fn subscribe(&mut self) -> bool {
        if let Ok(socket) = TincStream::subscribe(&self.tinc_pid) {
            self.socket = Some(socket);
            true
        }
        else {
            false
        }
    }

    fn connect_status_change(&self, is_connected: bool) {
        if is_connected {
            let _ = self.daemon_event_tx.send(DaemonEvent::TunnelConnected);
        }
        else {
            let _ = self.daemon_event_tx.send(DaemonEvent::TunnelDisconnected);
        }
    }

    fn recv_parse(&self, res: &str) -> Option<HostStatusChange> {
        let res: Vec<&str> = res.split_ascii_whitespace().collect();
        if res.len() == 4 {
            let event_str = res[2];
            let hosts = res[3];

            let host_status_change = match event_str {
                "Host-up" => HostStatusChange::HostUp(hosts.to_owned()),
                "Host-Down" => HostStatusChange::HostDown(hosts.to_owned()),
                _ => return None,
            };

            return Some(host_status_change);
        }
        else {
            return None;
        }
    }

    fn host_status_change_handle(&self, host_status_change: HostStatusChange) {
        let run_mode = &get_settings().common.mode;
        if *run_mode == RunMode::Center {
            std::thread::spawn(move || {
                let rpc_client = RpcClient::new();

                info!("{:?}", host_status_change);

                if let Err(e) = rpc_client.center_update_tinc_status(host_status_change) {
                    error!("{:?}", e);
                }
            });
        }
        else if *run_mode == RunMode::Client {
            match &host_status_change {
                HostStatusChange::HostUp(host) => {
                    if let Some(vip) = TincTools::get_vip_by_filename(host) {
                        add_route(&vip, 32, TINC_INTERFACE);
                    }
                    else {
                        error!("add_route - Can not parse host:{} to vip", host)
                    }
                }
                HostStatusChange::HostDown(host) => {
                    if let Some(vip) = TincTools::get_vip_by_filename(host) {
                        del_route(&vip, 32, TINC_INTERFACE);
                    }
                    else {
                        error!("del_route - Can not parse host:{} to vip", host)
                    }
                }
                _ => ()
            }
            #[cfg(windows)]
            windows::send_host_change(&host_status_change);
        }
    }
}

impl Drop for TincEventHandle {
    fn drop(&mut self) {
        if let Some(socket) = &self.socket {
            let _ = socket.shutdown(Shutdown::Both);
        }
    }
}

#[cfg(windows)]
mod windows {
    extern crate windows_named_pipe;
    use std::io::Write;

    use serde_json;
    use dnet_types::tinc_host_status_change::HostStatusChange;
    use crate::info::get_info;

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct TincEvent {
        state:              String,
        device_name:        String,
        team_id:            Vec<String>,
    }

    impl TincEvent {
        fn new(state: &str, host: &str) -> Option<Self> {
            if host.contains("proxy") {
                return Some(TincEvent {
                    state: state.to_owned(),
                    device_name: host.to_owned(),
                    team_id: vec![],
                })
            }

            let info = get_info().lock().unwrap();
            let (device_name, team_id) = info.teams.find_host_in_running(host);
            std::mem::drop(info);
            if device_name.len() > 0 && team_id.len() > 0 {
                let tinc_event = Self {
                    state: state.to_owned(),
                    device_name,
                    team_id,
                };
                Some(tinc_event)
            }
            else {
                None
            }
        }
    }

    pub fn send_host_change(host_status_change: &HostStatusChange) {
        match host_status_change {
            HostStatusChange::HostUp(host) => {
                if let Some(tinc_event) = TincEvent::new("host-up", host) {
                    if let Ok(buf) = serde_json::to_string(&tinc_event) {
                        ipc_sender(&buf)
                    }
                }
            }
            HostStatusChange::HostDown(host) => {
                if let Some(tinc_event) = TincEvent::new("host-down", host) {
                    if let Ok(buf) = serde_json::to_string(&tinc_event) {
                        ipc_sender(&buf)
                    }
                }
            }
            _ => ()
        }
    }

    fn ipc_sender(buf: &str) {
        if let Ok(mut _pipe) = windows_named_pipe::PipeStream::connect(
            r"\\.\pipe\dnet_client") {
            let _ = _pipe.write(buf.as_bytes());
        };
    }
}