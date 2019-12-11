use clap::{App, Arg, value_t_or_exit};
use crate::{new_ipc_client, Command};
use crate::error::Result;

pub struct Disconnect;

impl Command for Disconnect {
    fn name(&self) -> &'static str {
        "disconnect"
    }

    fn clap_subcommand(&self) -> App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .args(&[
                Arg::with_name("team_id"),
            ])
            .about("Try to disconnect if connected, or do nothing if already disconnected.")
    }
    // TODO disconnect all
    fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let team_id: String = value_t_or_exit!(matches.value_of("team_id"), String);
        let mut ipc = new_ipc_client()?;
        if let Err(e) = ipc.tunnel_disconnect(team_id) {
            eprintln!("{:?}", e);
        }
        Ok(())
    }
}
