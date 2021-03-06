use clap::App;
use crate::{new_ipc_client, Command};
use crate::error::{Result, Error};

pub struct Connect;

impl Command for Connect {
    fn name(&self) -> &'static str {
        "connect"
    }

    fn clap_subcommand(&self) -> App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Connect to all connected group.")
    }

    fn run(&self, _matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        let res = ipc.tunnel_connect()
            .map_err(Error::ipc_connect_failed)?;
        println!("{:#?}", res);
        Ok(())
    }
}
