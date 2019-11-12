use clap::App;
use crate::{new_ipc_client, Command};
use crate::error::Result;

pub struct Connect;

impl Command for Connect {
    fn name(&self) -> &'static str {
        "connect"
    }

    fn clap_subcommand(&self) -> App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Try to connect if disconnected, or do nothing if already connecting/connected.")
    }

    fn run(&self, _matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        match ipc.tunnel_connect() {
            Ok(x) => println!("{:?}", x),
            Err(e) => println!("{:?}", e),
        }
        Ok(())
    }
}
