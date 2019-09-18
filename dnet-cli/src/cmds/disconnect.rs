use clap::App;
use crate::{new_ipc_client, Command};
use crate::error::Result;

pub struct DisConnect;

impl Command for DisConnect {
    fn name(&self) -> &'static str {
        "disconnect"
    }

    fn clap_subcommand(&self) -> App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Try to disconnect if connected, or do nothing if already disconnected.")
    }

    fn run(&self, _matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        if let Err(e) = ipc.tunnel_disconnect() {
            eprintln!("{:?}", e);
        }
        Ok(())
    }
}
