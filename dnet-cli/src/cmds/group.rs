use clap::App;
use clap::value_t_or_exit;
use prettytable::{Table, Cell, Attr, color};

use crate::{new_ipc_client, Command};
use crate::error::{Error, Result};
use dnet_types::team::Team;

pub struct Group;

impl Command for Group {
    fn name(&self) -> &'static str {
        "group"
    }

    fn clap_subcommand(&self) -> App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Group operations.")
            .setting(clap::AppSettings::SubcommandRequiredElseHelp)
            .subcommand(
                clap::SubCommand::with_name("info")
                    .about("Display information about the currently of group info.")
                    .arg(
                        clap::Arg::with_name("team_id")
                            .help("Dnet team id.")
                            .required(true),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("users")
                    .about("Display information about the currently of group users.")
                    .arg(
                        clap::Arg::with_name("team_id")
                            .help("Dnet team id.")
                            .required(true),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("join")
                    .about("This device join group.")
                    .arg(
                        clap::Arg::with_name("team_id")
                            .help("Dnet team id.")
                            .required(true),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("out")
                    .about("This device out group.")
                    .arg(
                        clap::Arg::with_name("team_id")
                            .help("Dnet team id.")
                            .required(true),
                    ),
            )
            .subcommand(
                clap::SubCommand::with_name("list")
                    .about("List of all the group infos."),
            )
    }

    fn run(&self, matches: &clap::ArgMatches<'_>) -> Result<()> {
        if let Some(_matches) = matches.subcommand_matches("list") {
            self.group_list()?;
        } else if let Some(set_matches) = matches.subcommand_matches("info") {
            let team_id = value_t_or_exit!(set_matches.value_of("team_id"), String);
            self.group_info(team_id)?;
        } else if let Some(set_matches) = matches.subcommand_matches("users") {
            let team_id = value_t_or_exit!(set_matches.value_of("team_id"), String);
            self.group_users(team_id)?;
        } else if let Some(set_matches) = matches.subcommand_matches("join") {
            let team_id = value_t_or_exit!(set_matches.value_of("team_id"), String);
            self.group_join(team_id)?;
        } else if let Some(set_matches) = matches.subcommand_matches("out") {
            let team_id = value_t_or_exit!(set_matches.value_of("team_id"), String);
            self.group_out(team_id)?;
        } else {
            unreachable!("No account command given");
        }
        Ok(())
    }
}

impl Group {
    fn group_list(&self) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        let res = ipc.group_list()
            .map_err(Error::ipc_connect_failed)?;
        print_team(res);
        Ok(())
    }

    fn group_info(&self, team_id: String) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        let res = ipc.group_info(team_id)
            .map_err(Error::ipc_connect_failed)?;
        print_team(res);
        Ok(())
    }

    fn group_users(&self, team_id: String) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        let res = ipc.group_users(team_id)
            .map_err(Error::ipc_connect_failed)?;
        println!("{:#?}", res);
        Ok(())
    }

    fn group_join(&self, team_id: String) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        let res = ipc.group_join(team_id)
            .map_err(Error::ipc_connect_failed)?;
        println!("{:#?}", res);
        Ok(())
    }

    fn group_out(&self, team_id: String) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        let res = ipc.group_out(team_id)
            .map_err(Error::ipc_connect_failed)?;
        println!("{:#?}", res);
        Ok(())
    }
}

fn print_team(teams: Vec<Team>) {
    // Create the table
    let mut table = Table::new();
    // Add a row per time
    table.add_row(row!["Team Name", "Team Id", "Members Ip",
                                "Alias", "Connect Status", "Tunnel Status"]);
    let mut i = 0;
    for mut team in teams {
        if i == 0 {
            i = 1;
        }
        else {
            table.add_row(row![""]);
        }

        team.members.sort_by(|a, b|a.vip.cmp(&b.vip));
        for member in team.members {
            let connect_status = if member.connect_status {
                Cell::new("connect")
                    .with_style(Attr::ForegroundColor(color::GREEN))
            }
            else {
                Cell::new("disconnect")
                    .with_style(Attr::ForegroundColor(color::RED))
            };


            let tinc_status = if member.tinc_status {
                Cell::new("connect")
                    .with_style(Attr::ForegroundColor(color::GREEN))
            }
            else {
                Cell::new("disconnect")
                    .with_style(Attr::ForegroundColor(color::RED))
            };

            table.add_row(row![
                    team.team_name.clone().unwrap_or("".to_string()),
                    team.team_id,
                    member.vip,
                    member.device_name.unwrap_or("".to_string()),
                    connect_status,
                    tinc_status,
                ]);
        }
    }

    // Print the table to stdout
    table.printstd();
}