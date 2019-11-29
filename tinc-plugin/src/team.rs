use std::collections::HashMap;
use std::io::Write;
use crate::{TincStream, TincTools, TincOperatorError};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TincTeam {
    pub add: HashMap<String, Vec<String>>,
    pub delete: HashMap<String, Vec<String>>
}

impl TincTeam {
    pub fn new() -> Self {
        Self {
            add:        HashMap::new(),
            delete:     HashMap::new(),
        }
    }

    pub fn from_json_str(json: &str) -> std::result::Result<Self, serde_json::error::Error> {
        serde_json::from_str(json)
    }

    pub fn send_to_tinc(self, pid_file: &str) -> std::result::Result<(), Vec<String>> {
        let mut tinc_stream = match TincStream::new(pid_file) {
            Ok(x) => x,
            Err(e) => {
                error!("{:?}", e);

                let mut  failed = Self::get_keys(&self.add);
                failed.append(Self::get_keys(&self.delete).as_mut());

                return Err(failed);
            },
        };

        let mut failed = vec![];
        match tinc_stream.add_group_node(&self.add) {
            Ok(res) => {
                if let Err(mut failed_groups) = res {
                    failed.append(failed_groups.as_mut());
                }
            },
            Err(_) => {
                failed.append(self.add.keys()
                    .collect::<Vec<&String>>()
                    .into_iter()
                    .map(|keys|keys.to_owned())
                    .collect::<Vec<String>>()
                    .as_mut()
                )
            }
        }

        for (team_id, team_members) in self.delete {
            if team_members.is_empty() {
                if let Err(_) = tinc_stream.del_group(&team_id) {
                    failed.push(team_id);
                }
            }
            else {
                let mut nodes = String::new();
                for member in &team_members {
                    if nodes.is_empty() {
                        nodes = nodes + member;
                    }
                    else {
                        nodes = nodes + "," + member;
                    }
                }
                if let Err(_) = tinc_stream.del_group_node(&team_id, &nodes) {
                    failed.push(team_id);
                }
            }
        }
        if !failed.is_empty() {
            Err(failed)
        }
        else {
            Ok(())
        }
    }

    pub fn set_tinc_init_file(&self, path: &str) -> std::result::Result<(), TincOperatorError> {
        let mut buf = String::new();
        for (team_id, members) in &self.add {
            let mut members_str = String::new();
            for member in members {
                let name = TincTools::get_filename_by_vip(false, member);
                if members_str.len() != 0 {
                    members_str += ",";
                }
                members_str += &name;
            }

            buf += &format!("Group = {} {}\n", team_id, &members_str);
        }

        let mut file = std::fs::File::create(path.clone())
            .map_err(|e|
                TincOperatorError::IoError(path.to_string() + " " + &(e.to_string())))?;
        file.write(buf.as_bytes())
            .map_err(|e|
                TincOperatorError::IoError(path.to_string() + " " + &(e.to_string())))?;
        Ok(())
    }

    fn get_keys(hash: &HashMap<String, Vec<String>>) -> Vec<String> {
        hash.keys()
            .collect::<Vec<&String>>()
            .into_iter()
            .map(|keys|keys.to_owned())
            .collect::<Vec<String>>()
    }
}