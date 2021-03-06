use std::env;
use std::path::Path;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use management_client::{DaemonRpcClient, new_standalone_ipc_client};

#[derive(Serialize, Deserialize)]
enum HostStatusChange {
    TincUp,
    TincDown,
    HostUp(String),
    HostDown(String),
}

impl HostStatusChange {
    fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}

fn help() {
    let buf = "\r
    USAGE:\r
          mullvad <FLAGS>\r
    FLAGS:\r
        -h,                 Prints help information\r
        -u,                 Tinc Up\r
        -d,                 Tinc Down\r
        -hu <hostname>,     Host Up\r
        -hd <hostname>,     Host Down";
    println!("{}", buf);
}

pub fn new_ipc_client() -> DaemonRpcClient {
    let path = dnet_path::ipc_path();

    match new_standalone_ipc_client(&Path::new(&path)) {
        Ok(client) => client,
        Err(e) => {
            panic!(format!("{:?}", e));
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let host_status_change;
        match &args[1][..] {
            "-u" => host_status_change = HostStatusChange::TincUp,
            "-d" => host_status_change = HostStatusChange::TincDown,
            "-hu" => {
                if args.len() > 2 {
                    let host = args[2].to_owned();
                    if !host.contains("proxy") {
                        host_status_change = HostStatusChange::HostUp(args[2].to_owned());
                    }
                    else {
                        std::process::exit(0);
                    }
                }
                else {
                    help();
                    std::process::exit(1);
                }
            },
            _ if args[1] == "-hd" => {
                if args.len() > 2 {
                    let host = args[2].to_owned();
                    if !host.contains("proxy") {
                        host_status_change = HostStatusChange::HostDown(args[2].to_owned());
                    }
                    else {
                        std::process::exit(0);
                    }
                }
                else {
                    help();
                    std::process::exit(1);
                }
            },
            _ => {
                help();
                std::process::exit(1);
            }
        }

        let mut client = new_ipc_client();
        let _ = client.host_status_change(host_status_change.to_json());
    }
    else {
        help();
    }
}