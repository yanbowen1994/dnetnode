use std::collections::HashMap;


extern crate talpid_ipc;
#[macro_use]
extern crate serde_derive;
use ipc_client::{new_standalone_ipc_client, DaemonRpcClient};
use clap::App;

use serde::{Serialize, Deserialize};

mod cmds;
mod error;
use cmds::rpc::Rpc;
use error::{Error, Result};
use std::path::Path;

pub const COMMIT_ID: &str = include_str!(concat!(env!("OUT_DIR"), "/git-commit-id.txt"));

pub const COMMIT_DATE: &str = include_str!(concat!(env!("OUT_DIR"), "/git-commit-date.txt"));

pub fn new_ipc_client() -> Result<DaemonRpcClient> {
    // TODO dnet path
    match new_standalone_ipc_client(&Path::new(&"/opt/dnet/dnet.socket".to_string())) {
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
