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
    pub fn new() -> Self {
        Self {
            ip:     IpAddr::from_str("255.255.255.255").unwrap(),
            vip:    IpAddr::from_str("255.255.255.255").unwrap(),
            pubkey: String::new(),
        }
    }

    pub fn from(ip:IpAddr, vip:IpAddr, pubkey:String) -> Self {
        Self {
            ip,
            vip,
            pubkey,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TincInfo {
    pub ip:         Option<IpAddr>,
    pub vip:        IpAddr,
    pub pub_key:    String,
    pub mode:       TincRunMode,
    pub connect_to: Vec<ConnectTo>,
}

impl TincInfo {
    pub fn new() -> Self {
        let ip = None;
        let vip = IpAddr::from_str("255.255.255.255").unwrap();
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
