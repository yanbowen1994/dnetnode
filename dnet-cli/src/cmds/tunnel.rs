use clap::App;
use crate::{new_ipc_client, Command};
use crate::error::{Error, Result};

pub struct Tunnel;

impl Command for Tunnel {
    fn name(&self) -> &'static str {
        "tunnel"
    }

    fn clap_subcommand(&self) -> App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Tunnel connect or disconnect.")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("connect")
                    .about("Tunnel connect to proxy.")
            )
            .subcommand(
                clap::SubCommand::with_name("disconnect")
                    .about("Tunnel disconnect with proxy.")
            )
    }

    fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        if let Some(_) = matches.subcommand_matches("connect") {
            self.tunnel_connect()
        }

        else if let Some(_) = matches.subcommand_matches("disconnect") {
            self.tunnel_disconnect()
        }

        else {
            unreachable!("No tunnel command given");
        }
    }
}

impl Tunnel {
    fn tunnel_connect(&self) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        ipc.tunnel_connect()
            .map_err(Error::ipc_connect_failed)?;
        println!("Tunnel connecting.");
        Ok(())
    }

    fn tunnel_disconnect(&self) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        ipc.tunnel_disconnect()
            .map_err(Error::ipc_connect_failed)?;
        println!("Tunnel disconnecting.");
        Ok(())
    }
}