use std::io;

use tinc_plugin::{PID_FILENAME, TincStream};

use dnet_types::tinc_host_status_change::HostStatusChange;
use crate::settings::get_settings;
use crate::rpc::proxy::RpcClient;
use std::net::Shutdown;

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

    fn subscribe(&mut self) {
        self.socket = TincStream::subscribe(&self.tinc_pid).ok();
    }

    pub fn recv(&mut self) {
        if let None = self.socket {
            self.subscribe();
        }

        if let Some(socket) = &self.socket {
            match TincStream::recv_from_subscribe(socket) {
                Ok(res) => {
                    recv_parse(&res);
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
}

impl Drop for TincEventHandle {
    fn drop(&mut self) {
        if let Some(socket) = &self.socket {
            let _ = socket.shutdown(Shutdown::Both);
        }
    }
}

fn recv_parse(res: &str) {
    let res: Vec<&str> = res.split_ascii_whitespace().collect();
    if res.len() == 4 {
        let event_str = res[2];
        let hosts = res[3];

        let host_status_change = match event_str {
            "Host-up" => HostStatusChange::HostUp(hosts.to_owned()),
            "Host-Down" => HostStatusChange::HostDown(hosts.to_owned()),
            _ => return,
        };

        std::thread::spawn(move || {
            let rpc_client = RpcClient::new();

            info!("{:?}", host_status_change);

            if let Err(e) = rpc_client.center_update_tinc_status(host_status_change) {
                error!("{:?}", e);
            }
        });
    }
    else {
        return;
    }
}
