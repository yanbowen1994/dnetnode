use std::collections::HashMap;
use crate::TincStream;

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

    pub fn send_to_tinc(self, pid_file: &str) -> std::result::Result<(), Self> {
        let mut tinc_stream = match TincStream::new(pid_file) {
            Ok(x) => x,
            Err(_) => return Err(self),
        };

        let mut failed = Self::new();

        for (team_id, team_members) in self.add {
            let mut nodes = String::new();
            for member in &team_members {
                if nodes.is_empty() {
                    nodes = nodes + member;
                }
                else {
                    nodes = nodes + "," + member;
                }
            }
            if let Err(_) = tinc_stream.add_group_node(&team_id, &nodes) {
                failed.add.insert(team_id, team_members);
            }
        }

        for (team_id, team_members) in self.delete {
            if team_members.is_empty() {
                if let Err(_) = tinc_stream.del_group(&team_id) {
                    failed.delete.insert(team_id, vec![]);
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
                    failed.delete.insert(team_id, team_members);
                }
            }
        }
        if !failed.add.is_empty() || !failed.delete.is_empty() {
            Err(failed)
        }
        else {
            Ok(())
        }
    }

//    pub fn set_tinc_init_file(&self) {
//        let node_info = team_info.parse_team_to_node_info();
//
//        let mut buf = String::new();
//        for (node, sub_nodes) in node_info {
//            let mut sub_nodes_str = String::new();
//            for sub_node in sub_nodes {
//                let sub_node_file_name = TincTools::get_filename_by_vip(false, &sub_node.0);
//                if sub_nodes_str.len() == 0 {
//                    sub_nodes_str += &sub_node_file_name;
//                }
//                else {
//                    sub_nodes_str += "#";
//                    sub_nodes_str += &sub_node_file_name;
//                }
//            }
//            let node = TincTools::get_filename_by_vip(false, &node);
//            buf += &format!("VLAN = {} {}\n", &node, &sub_nodes_str);
//        }
//
//        let path = self.tinc_home.clone() +  "/tinc.vlan";
//
//        let mut file = fs::File::create(path.clone())
//            .map_err(|e|TincOperatorError::IoError(path.clone() + " " + &e.to_string()))?;
//        file.write(buf.as_bytes())
//            .map_err(|e|TincOperatorError::IoError(path.clone() + " " + &e.to_string()))?;
//
//        return Ok(());
//    }
}