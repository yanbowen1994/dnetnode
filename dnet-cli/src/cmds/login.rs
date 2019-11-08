use clap::{App, Arg};
use clap::value_t_or_exit;

use crate::{new_ipc_client, Command};
use crate::error::Result;
use dnet_types::user::User;

pub struct Login;

impl Command for Login {
    fn name(&self) -> &'static str {
        "login"
    }

    fn clap_subcommand(&self) -> App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .args(&[
                Arg::with_name("user name"),
                Arg::with_name("password"),
            ])
            .about("Set login user.")
    }

    fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        let name = value_t_or_exit!(matches.value_of("user name"), String);
        let password = value_t_or_exit!(matches.value_of("password"), String);

        let user = User::new(&name, &password).to_json_str();

        println!("{:?}", user);

        let mut ipc = new_ipc_client()?;
        match ipc.login(user) {
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
