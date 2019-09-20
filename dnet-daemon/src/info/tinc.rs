use std::str::FromStr;
use std::net::IpAddr;
use std::fs::File;
use std::io::Read;

use tinc_plugin::{TincOperatorError, PUB_KEY_FILENAME};

use crate::settings::get_settings;
use crate::tinc_manager::{TincOperator, tinc_connections};

use super::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct TincInfo {
    pub vip:            IpAddr,
    pub pub_key:        String,
    pub connections:    u32,
    pub edges:          u32,
    pub nodes:          u32,
    tinc_home:          String,
}
impl TincInfo {
    pub fn new() -> Self {
        let tinc_home = &get_settings().common.home_path.clone()
            .join("tinc").to_str().unwrap().to_string();
        let vip = IpAddr::from_str("10.255.255.254").unwrap();
        let pub_key = "".to_owned();
        TincInfo {
            vip,
            pub_key,
            connections:    0,
            edges:          0,
            nodes:          0,
            tinc_home:      tinc_home.to_owned(),
        }
    }

    // Load local tinc config file vpnserver for tinc vip and pub_key.
    // Success return true.
    pub fn load_local(&mut self) {
        if let Ok(vip) = self.load_local_vip() {
            self.vip = vip;
        }
        self.pub_key = self.load_local_pubkey()
            .expect("Must create tinc key pair before Info init.");
    }

    fn load_local_vip(&self) -> Result<IpAddr> {
        let tinc = TincOperator::new();
        tinc.get_vip().map_err(Error::GetVip)
    }

    pub fn flush_connections(&mut self)
                             -> Result<()> {
        let (connections, edges, nodes) = tinc_connections (
            &(self.tinc_home.to_string() + "/tinc.pid"))
            .map_err(Error::TincDump)?;
        self.connections = connections;
        self.edges = edges;
        self.nodes = nodes;
        Ok(())
    }

    fn load_local_pubkey(&self) -> Result<String> {
        let settings = get_settings();
        let tinc_home = settings.common.home_path.clone()
            .join("tinc").join(PUB_KEY_FILENAME).to_str().unwrap().to_string();
        let mut res = String::new();
        let mut _file = File::open(tinc_home)
            .map_err(Error::FileNotExit)?;
        _file.read_to_string(&mut res)
            .map_err(Error::FileNotExit)?;
        Ok(res)
    }
}
