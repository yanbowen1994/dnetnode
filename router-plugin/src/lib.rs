use std::process::Command;

#[macro_use]
extern crate serde_derive;

pub mod device_info;
//mod team_status_response;

pub fn get_sn() -> Option<String> {
    let out = Command::new("artmtd").arg("-r").arg("sn").output().unwrap();
    let res = String::from_utf8( out.stdout).unwrap_or(String::new());
    let res: Vec<&str> = res.split("\n").collect();
    if res.is_empty() {
        return None
    }
    else {
        let res = res[0].replace("sn:", "");
        Some(res)
    }
}