extern crate ipc_server;

use ipc_client::{new_standalone_ipc_client, DaemonRpcClient};
use clap::App;

mod cmds;
mod error;
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
    let commands = cmds::get_commands();

    let matches =  App::new("dnet")
        .version(&format!("\nCommit date: {}\nCommit id: {}", COMMIT_DATE, COMMIT_ID).to_string()[..])
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommands(commands.values().map(|cmd| cmd.clap_subcommand()))
        .get_matches();

    let (subcommand_name, subcommand_matches) = matches.subcommand();
    if let Some(cmd) = commands.get(subcommand_name) {
        cmd.run(subcommand_matches.expect("No command matched"))
            .map_err(|e|{
                println!("{:?}", e);
                ()
            })
            .unwrap_or(());
    }
}

pub trait Command {
    fn name(&self) -> &'static str;

    fn clap_subcommand(&self) -> clap::App<'static, 'static>;

    fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()>;
}

