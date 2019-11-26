use std::time::Duration;

use tinc_plugin::{PID_FILENAME, TincStream};

use crate::settings::get_settings;
use crate::rpc::proxy::RpcClient;
use dnet_types::tinc_host_status_change::HostStatusChange;

pub fn tinc_event_recv() {
    let tinc_pid = get_settings().common.home_path
        .join("tinc").join(PID_FILENAME)
        .to_str().unwrap().to_string();

    loop {
        if let Err(e) = TincStream::subscribe(&tinc_pid, recv_parse) {
            error!("{:?}", e);
        }

        std::thread::sleep(Duration::from_secs(1));
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

        let rpc_client = RpcClient::new();

        info!("{:?}", host_status_change);

        if let Err(e) = rpc_client.center_update_tinc_status(host_status_change) {
            error!("{:?}", e);
        }
    }
    else {
        return;
    }
}