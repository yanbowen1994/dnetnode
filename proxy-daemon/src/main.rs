#[macro_use]
extern crate log;

extern crate common_core;
extern crate proxy_daemon;

use common_core::daemon::Daemon;
use proxy_daemon::domain::Info;
use proxy_daemon::http_server_client::RpcMonitor;
use proxy_daemon::tinc_manager::TincMonitor;

fn main() {
    let exit_code = match common_core::init() {
        Ok(_) => {
            start_daemon();
            0
        },
        Err(error) => {
            println!("{:?}\n{}", error, error);
            1
        }
    };

    debug!("Process exiting with code {}", exit_code);
    std::process::exit(exit_code);
}

fn start_daemon() {
    let mut daemon = Daemon::<Info>::start::<RpcMonitor, TincMonitor>();
    daemon.run();
}