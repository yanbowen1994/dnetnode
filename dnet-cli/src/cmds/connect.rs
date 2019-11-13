use clap::{App, Arg};
use crate::{new_ipc_client, Command};
use crate::error::Result;

pub struct Connect;

impl Command for Connect {
    fn name(&self) -> &'static str {
        "connect"
    }

    fn clap_subcommand(&self) -> App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .args(&[
                Arg::with_name("team_id"),
            ])
            .about("Try to connect if disconnected, or do nothing if already connecting/connected.")
    }

    fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let team_id: String = value_t_or_exit!(matches.value_of("team_id"), String);
        let mut ipc = new_ipc_client()?;
        match ipc.tunnel_connect(team_id) {
            Ok(x) => println!("{:?}", x),
            Err(e) => println!("{:?}", e),
        }
        Ok(())
    }
}
