use std::collections::HashMap;
use clap::App;

#[macro_use]
extern crate serde_derive;
use talpid_ipc::client::{new_standalone_ipc_client, DaemonRpcClient};

mod cmds;
mod error;
use cmds::rpc::Rpc;
use error::*;

mod ipc_client;

pub const COMMIT_ID: &str = include_str!(concat!(env!("OUT_DIR"), "/git-commit-id.txt"));

pub const COMMIT_DATE: &str = include_str!(concat!(env!("OUT_DIR"), "/git-commit-date.txt"));

pub fn new_rpc_client() -> Result<DaemonRpcClient> {
    match new_standalone_ipc_client(&mullvad_paths::get_rpc_socket_path()) {
        Err(e) => Err(Error::DaemonNotRunning(e)),
        Ok(client) => Ok(client),
    }
}

fn main() {
    let mut commands = HashMap::new();
    commands.insert(Rpc{}.name(), Rpc{});
    let matches =  App::new("dnet")
        .version(&format!("\nCommit date: {}\nCommit id: {}", COMMIT_DATE, COMMIT_ID).to_string()[..])
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommands(commands.values().map(|cmd| cmd.clap_subcommand()))
        .get_matches();

    let (subcommand_name, subcommand_matches) = matches.subcommand();
    if let Some(cmd) = commands.get(subcommand_name) {
        cmd.run(subcommand_matches.expect("No command matched"))
    }
}
