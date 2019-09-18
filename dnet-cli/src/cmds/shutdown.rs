use clap::App;
use crate::{new_ipc_client, Command};
use crate::error::Result;

pub struct Shutdown;

impl Command for Shutdown {
    fn name(&self) -> &'static str {
        "shutdown"
    }

    fn clap_subcommand(&self) -> App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Stop daemon.")
    }

    fn run(&self, _matches: &clap::ArgMatches<'_>) -> Result<()> {
        let mut ipc = new_ipc_client()?;

        match ipc.shutdown() {
            Ok(res) => {
                if res.code == 200 {
                    println!("Daemon shutdown.")
                }
                else {
                    println!("Daemon shutdown failed.")
                }
            }
            Err(e) => println!("{:?}", e),
        }
        Ok(())
    }
}
