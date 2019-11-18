use clap::App;

use crate::{new_ipc_client, Command};
use crate::error::Result;

pub struct Logout;

impl Command for Logout {
    fn name(&self) -> &'static str {
        "logout"
    }

    fn clap_subcommand(&self) -> App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("logout.")
    }

    fn run(&self, _matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        match ipc.logout() {
            Ok(res) => {
                println!("Response {:?}", res);
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
        Ok(())
    }
}
