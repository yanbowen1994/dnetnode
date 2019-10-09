mod team_status_response;

use std::process::Command;

pub fn get_sn() -> Option<String> {
    let out = Command::new("artmtd").arg("-r").arg("sn").output().unwrap();
    let res = String::from_utf8_lossy( &out.stdout).to_string();
    let res: Vec<&str> = res.split("\n").collect();
    if res.is_empty() {
        return None
    }
    else {
        let res = res[0].replace("sn:", "");
        Some(res)
    }
}