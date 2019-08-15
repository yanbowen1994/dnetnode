use std::net::IpAddr;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TincRunMode {
    Client,
    Proxy,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ConnectTo {
    pub ip:                 IpAddr,
    pub vip:                IpAddr,
    pub pubkey:             String,
}
impl ConnectTo {
    pub fn new(ip:IpAddr, vip:IpAddr, pubkey:String) -> Self {
        Self {
            ip,
            vip,
            pubkey,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TincInfo {
    pub ip:         IpAddr,
    pub vip:        IpAddr,
    pub pub_key:    String,
    pub mode:       TincRunMode,
    pub connect_to: Vec<ConnectTo>,
}

impl TincInfo {
    pub fn new() -> Self {
        let ip = IpAddr::from_str("0.0.0.0").unwrap();
        let vip = IpAddr::from_str("0.0.0.0").unwrap();
        let pub_key = "".to_string();
        TincInfo {
            ip,
            vip,
            pub_key,
            mode: TincRunMode::Client,
            connect_to: vec![],
        }
    }
}
