use clap::App;

use crate::{new_ipc_client, Command};
use crate::error::{Error, Result};

pub struct Status;

impl Command for Status {
    fn name(&self) -> &'static str {
        "status"
    }

    fn clap_subcommand(&self) -> App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Daemon status.")
    }

    fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        self.status()
    }
}

impl Status {
    fn status(&self) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        let res = ipc.status()
            .map_err(Error::ipc_connect_failed)?;
        println!("{:#?}", res);
        Ok(())
    }
}