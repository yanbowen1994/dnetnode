use clap::App;
use clap::value_t_or_exit;
use prettytable::{Table, Cell};

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
                clap::SubCommand::with_name("quit")
                    .about("This device quit group.")
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
        } else if let Some(set_matches) = matches.subcommand_matches("quit") {
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
        if let Some(teams_json) = res.data.clone() {
            if let Ok(teams) = serde_json::from_value::<Vec<Team>>(teams_json) {
                print_team(teams);
            }
            else {
                println!("Can't parse response. {:#?}", res);
            }
        }
        else {
            println!("{:#?}", res);
        }
        Ok(())
    }

    fn group_info(&self, team_id: String) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        let res = ipc.group_info(team_id)
            .map_err(Error::ipc_connect_failed)?;
        if let Some(teams_json) = res.data.clone() {
            if let Ok(teams) = serde_json::from_value::<Vec<Team>>(teams_json) {
                print_team(teams);
            }
            else {
                println!("Can't parse response. {:#?}", res);
            }
        }
        else {
            println!("{:#?}", res);
        }
        Ok(())
    }

    fn group_users(&self, team_id: String) -> Result<()> {
        let mut ipc = new_ipc_client()?;
        let res = ipc.group_users(team_id)
            .map_err(Error::ipc_connect_failed)?;
        print_user(res.data);
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

fn print_team(mut teams: Vec<Team>) {
    // Create the table
    let mut table = Table::new();
    // Add a row per time
    table.set_titles(row!["Team Name", "Team Id", "Members Ip", "Self",
                                "Alias", "Status", "Team Status", "Proxy Status"]);
    teams.sort_by(|a, b|a.team_name.cmp(&b.team_name));
    for mut team in teams {
        team.members.sort_by(|a, b|a.vip.cmp(&b.vip));
        if team.members.len() == 0 {
            table.add_row(row![
                            team.team_name.clone().unwrap_or("".to_string()),
                            team.team_id,
                            "",
                            "",
                            "",
                            "",
                            "",
                            "",
                        ]);
        }
        else {
            for member in team.members {
                let connect_status =
                    if member.connect_status {
                        Cell::new("connect")
                            .style_spec("Bg")
                    }
                    else {
                        Cell::new("disconnect")
                            .style_spec("Br")
                    };

                let tinc_status = if member.tinc_status {
                    Cell::new("connect")
                        .style_spec("Bg")
                }
                else {
                    Cell::new("disconnect")
                        .style_spec("Br")
                };

                let host_status  =
                    if let Some(host_status) =  member.is_local_tinc_host_up {
                        if host_status {
                            Cell::new("connect")
                                .style_spec("Bg")
                        }
                        else {
                            Cell::new("disconnect")
                                .style_spec("Br")
                        }
                    }
                    else {
                        Cell::new("disconnect")
                            .style_spec("Br")
                    };

                let is_self = match member.is_self {
                    Some(x) => {
                        if x {
                            "yes"
                        }
                        else {
                            ""
                        }
                    },
                    None => "",
                };

                table.add_row(row![
                            team.team_name.clone().unwrap_or("".to_string()),
                            team.team_id,
                            member.vip,
                            is_self,
                            member.device_name.unwrap_or("".to_string()),
                            host_status,
                            connect_status,
                            tinc_status,
                        ]);
            }
        }
    }

    // Print the table to stdout
    table.printstd();
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserInfo {
    pub name:              Option<String>,
    pub email:             Option<String>,
    pub photo:             Option<String>,
}

fn print_user(user_json: Option<serde_json::Value>) {
    let mut users = match user_json
        .and_then(|user_json| {
            serde_json::from_value::<Vec<serde_json::Value>>(user_json).ok()
        })
        .and_then(|user_json| {
            Some(user_json
                .into_iter()
                .filter_map(|user_json| {
                    serde_json::from_value::<UserInfo>(user_json).ok()
                })
                .collect::<Vec<UserInfo>>())
        }) {
        Some(x) => x,
        None => return,
    };

    // Create the table
    let mut table = Table::new();
    // Add a row per time
    table.add_row(row!["Account", "Email"]);
    users.sort_by(|a, b|a.name.cmp(&b.name));
    for user in users {
        table.add_row(row![
                            user.name.unwrap_or("".to_string()),
                            user.email.unwrap_or("".to_string()),
                        ]);
    }
    // Print the table to stdout
    table.printstd();
}