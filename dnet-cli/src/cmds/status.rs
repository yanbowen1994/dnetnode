use clap::App;

use crate::{new_ipc_client, Command};
use crate::error::{Error, Result};
use prettytable::Table;
use std::net::IpAddr;

pub struct Status;

impl Command for Status {
    fn name(&self) -> &'static str {
        "status"
    }

    fn clap_subcommand(&self) -> App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Daemon status.")
    }

    fn run(&self, _matches: &clap::ArgMatches<'_>) -> Result<()> {
        self.status()
    }
}

impl Status {
    fn status(&self) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        let res = ipc.status()
            .map_err(Error::ipc_connect_failed)?;
        if let Some(data) = res.data {
            if let Ok(data) = serde_json::from_value::<ResponseData>(data) {
                data.print();
            }
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct ResponseData {
    status: dnet_types::status::Status,
    vip:    Option<IpAddr>,
}

impl ResponseData {
    fn print(self) {
        let mut table = Table::new();
        table.add_row(row!["Tunnel", "Cloud", "Daemon", "Vip"]);
        table.add_row(row![
             format!("{:?}", self.status.tunnel),
             format!("{:?}", self.status.rpc),
             format!("{:?}", self.status.daemon),
             self.vip.map(|vip|vip.to_string()).unwrap_or("".to_string()),
        ]);
        table.printstd();
    }
}