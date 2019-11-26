use std::net::IpAddr;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TincRunMode {
    Center,
    Client,
    Proxy,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ConnectTo {
    pub id:                 String,
    pub ip:                 IpAddr,
    pub vip:                IpAddr,
    pub port:               u16,
    pub pubkey:             String,
}
impl ConnectTo {
    pub fn from(id: String, ip:IpAddr, vip:IpAddr, port: u16, pubkey:String) -> Self {
        Self {
            id,
            ip,
            vip,
            port,
            pubkey,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TincInfo {
    pub ip:         Option<IpAddr>,
    pub vip:        IpAddr,
    pub port:       u16,
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
            port: 50069,
            pub_key,
            mode: TincRunMode::Client,
            connect_to: vec![],
        }
    }
}
