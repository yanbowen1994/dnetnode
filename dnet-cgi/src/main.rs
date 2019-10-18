use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;

extern crate ipc_server;
#[macro_use]
extern crate serde_derive;

use ipc_client::{new_standalone_ipc_client, DaemonRpcClient};
use dnet_types::team::Team;
use router_plugin::team_status_response::TeamStatusResponse;

pub fn new_ipc_client() -> Option<DaemonRpcClient> {
    match new_standalone_ipc_client(&Path::new(&"/opt/dnet/dnet.socket".to_string())) {
        Ok(client) => Some(client),
        Err(_) => None,
    }
}

enum RequestMethod {
    Get(GetAction),
    Post,
}

#[allow(non_camel_case_types)]
enum GetAction {
    get_vpn_status,
}

struct InputHandle;

impl InputHandle {
    fn get_input() -> Option<RequestMethod> {
        if let Ok(method) = env::var("REQUEST_METHOD") {
            if &method == "GET" {
                if let Ok(env_input) = env::var("QUERY_STRING") {
                    if let Some(param) = Self::parse_get_args(&env_input) {
                        return Some(RequestMethod::Get(param));
                    }
                }
            } else {
                let input = Self::read_req_txt().unwrap_or("".to_owned());
                return Some(RequestMethod::Post);
            }
        }
        println!("REQUEST_METHOD env not found.");
        None
    }

    fn read_req_txt() -> Option<String> {
        if let Ok(mut file) = fs::File::open("/tmp/req.txt") {
            let mut contents = String::new();
            if let Ok(_) = file.read_to_string(&mut contents) {
                return Some(contents);
            }
        }
        None
    }

    fn parse_get_args(input: &str) -> Option<GetAction> {
        let mut action = None;

        let fields: Vec<&str> = input.split("&").collect();
        for field in fields {
            let kv: Vec<&str> = field.split("=").collect();
            if kv.len() > 1 {
                let key = kv[0];
                let value = kv[1];
                action = match key {
                    "get_vpn_status" => Some(GetAction::get_vpn_status),
                    _ => None,
                };
            }
        }
        return action;
    }
}

struct Exec;

impl Exec {
    fn get_vpn_status() {

    }
}

//fn main() {
//    if let Some(mut ipc) = new_ipc_client() {
//        if let Some(request) = InputHandle::get_input() {
//            match request {
//                RequestMethod::Get(GetAction::get_vpn_status) => {
//                    let teams = ipc.group_list().unwrap();
//                    let team_status_response = TeamStatusResponse::from(teams);
//                    let json = team_status_response.to_json_str();
//                    println!("{}", json)
//                },
//                RequestMethod::Post => (),
//            }
//        }
//    }
//}

fn main() {
    if let Some(mut ipc) = new_ipc_client() {
        let teams = ipc.group_list().unwrap();
        let team_status_response = TeamStatusResponse::from(teams);
        let json = team_status_response.to_json_str();
        println!("{}", json);
    }
    else {
        let json = TeamStatusResponse {
            code: 500,
            teams: vec![],
        }.to_json_str();
        println!("{}", json);
    }
}

