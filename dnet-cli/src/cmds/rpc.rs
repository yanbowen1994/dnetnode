use clap::App;
use crate::ipc_client::new_ipc_client;

pub struct Rpc;

impl Rpc {
    pub fn new() -> Self {
        Self {}
    }

    pub fn name(&self) -> &'static str {
        "rpc"
    }

    pub fn clap_subcommand(&self) -> App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Connection with cloud.")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("status")
                    .about("Get connection status.")
            )
    }

    pub fn run(&self, matches: &clap::ArgMatches<'_>) {
        if let Some(set_matches) = matches.subcommand_matches("status") {
            self.rpc_status();
        }
    }

    fn rpc_status(&self) {
        let mut ipc = new_ipc_client();
    }


}