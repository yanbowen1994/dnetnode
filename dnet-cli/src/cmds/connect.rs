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
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
    }

    fn run(&self, _matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        if let Err(e) = ipc.tunnel_connect() {
            eprintln!("{:?}", e);
        }
        Ok(())
    }
}
