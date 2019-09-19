use std::process::Command;

pub fn get_sn() -> String {
    let out = Command::new("artmtd").arg("-r").arg("sn").output().unwrap();
    let res = String::from_utf8_lossy( &out.stdout).to_string();
    let res: Vec<&str> = res.split("\n").collect();
    let res = res[0].replace("sn:", "");
    res
}