use std::io;

use tinc_plugin::{PID_FILENAME, TincStream, TincTools};

use dnet_types::tinc_host_status_change::HostStatusChange;
use crate::settings::get_settings;
use crate::rpc::proxy::RpcClient;
use std::net::Shutdown;
use dnet_types::settings::RunMode;
use sandbox::route::{add_route, del_route};
use crate::settings::default_settings::TINC_INTERFACE;

pub struct TincEventHandle {
    socket: Option<socket2::Socket>,
    tinc_pid: String,
}

impl TincEventHandle {
    pub fn new() -> Self {
        let tinc_pid = get_settings().common.home_path
            .join("tinc").join(PID_FILENAME)
            .to_str().unwrap().to_string();

        let mut tinc_event_handle = Self {
            socket:  None,
            tinc_pid,
        };
        tinc_event_handle.subscribe();
        tinc_event_handle
    }

    pub fn recv(&mut self) {
        if let None = self.socket {
            self.subscribe();
        }

        if let Some(socket) = &self.socket {
            match TincStream::recv_from_subscribe(socket) {
                Ok(res) => {
                    self.recv_parse(&res);
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::TimedOut {
                        return;
                    }
                    else {
                        self.socket = None;
                    }
                }
            }
        }
    }

    fn subscribe(&mut self) {
        self.socket = TincStream::subscribe(&self.tinc_pid).ok();
    }

    fn recv_parse(&self, res: &str) {
        let res: Vec<&str> = res.split_ascii_whitespace().collect();
        if res.len() == 4 {
            let event_str = res[2];
            let hosts = res[3];

            let host_status_change = match event_str {
                "Host-up" => HostStatusChange::HostUp(hosts.to_owned()),
                "Host-Down" => HostStatusChange::HostDown(hosts.to_owned()),
                _ => return,
            };

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
        else {
            return;
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