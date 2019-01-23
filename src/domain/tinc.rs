use std::str::FromStr;
use std::net::IpAddr;
use std::fs::{File, read_to_string};
use std::io::Read;

#[derive(Debug, Clone)]
pub struct TincInfo {
    pub vip: IpAddr,
    pub pub_key: String,
}
impl TincInfo {
    pub fn new() -> Self {
        let vip = IpAddr::from_str("0.0.0.0").unwrap();
        let pub_key = "".to_string();
        TincInfo {
            vip,
            pub_key,
        }
    }

    // Load local tinc config file vpnserver for tinc vip and pub_key.
    // Success return true.
    pub fn load_local(&mut self, tinc_home: &str, pub_key_path: &str) -> bool {
        let mut file = File::open(tinc_home.to_string() + "/hosts/vpnserver").unwrap_or(return false);
        let mut res = String::new();
        file.read_to_string(&mut res).unwrap_or(return false);
        let tmp: Vec<&str> = res.split("\n").collect();
        let tmp: Vec<&str> = tmp[0].split(" ").collect();
        let vip = tmp[2];
        self.vip= IpAddr::from_str(vip).unwrap_or(return false);

        let mut file = File::open(tinc_home.to_string() + pub_key_path).unwrap_or(return false);
        file.read_to_string(&mut res).unwrap_or(return false);
        self.pub_key = res;
        return true;
    }
}
