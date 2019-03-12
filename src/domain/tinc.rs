use std::str::FromStr;
use std::net::IpAddr;
use std::fs::File;
use std::io::Read;
use tinc_manager::operator::Tinc;

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
        {
            let mut res = String::new();
            let mut _file = File::open(tinc_home.to_string() + pub_key_path).unwrap();
            _file.read_to_string(&mut res).unwrap();
            self.pub_key = res.clone().replace("\n", "");
        }
        {
            let tinc = Tinc::new(tinc_home.to_string(), pub_key_path.to_string());
            self.vip = IpAddr::from_str(&tinc.get_vip()).unwrap();
        }
        return true;
    }
}
