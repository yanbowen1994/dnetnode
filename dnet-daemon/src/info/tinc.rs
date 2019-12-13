use std::net::IpAddr;
use std::fs::File;
use std::io::Read;

use tinc_plugin::{ConnectTo, PUB_KEY_FILENAME, PID_FILENAME, DEFAULT_TINC_PORT};

use crate::settings::get_settings;
use crate::tinc_manager::{TincOperator, tinc_connections};

use super::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct TincInfo {
    pub vip:                    Option<IpAddr>,
    pub pub_key:                String,
    pub port:                   u16,
    pub connections:            u32,
    pub edges:                  u32,
    pub nodes:                  u32,
    pub connect_to:             Vec<ConnectTo>,
    pub last_runtime:           Option<String>,
    pub current_connect:        Vec<IpAddr>,
    tinc_home:          String,
}
impl TincInfo {
    pub fn new() -> Self {
        let tinc_home = get_settings().common.home_path.clone()
            .join("tinc").to_str().unwrap().to_string() + "/";
        let pub_key = "".to_owned();
        TincInfo {
            vip:                    None,
            pub_key,
            port:                   DEFAULT_TINC_PORT,
            connections:            0,
            edges:                  0,
            nodes:                  0,
            last_runtime:           None,
            tinc_home,
            current_connect:        vec![],
            connect_to:             vec![],
        }
    }

    // Load local tinc config file vpnserver for tinc vip and pub_key.
    // Success return true.
    pub fn load_local(&mut self) {
        if let Ok(vip) = self.load_local_vip() {
            self.vip = Some(vip);
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
            &(self.tinc_home.to_string() + PID_FILENAME))
            .map_err(Error::TincDump)?;
        self.connections = connections;
        self.edges = edges;
        self.nodes = nodes;
        Ok(())
    }

    fn load_local_pubkey(&self) -> Result<String> {
        let settings = get_settings();
        let pubkey_file = settings.common.home_path.clone()
            .join("tinc").join(PUB_KEY_FILENAME).to_str().unwrap().to_string();
        let mut res = String::new();
        let mut _file = File::open(pubkey_file)
            .map_err(Error::FileNotExit)?;
        _file.read_to_string(&mut res)
            .map_err(Error::FileNotExit)?;
        Ok(res)
    }

    pub fn remove_current_connect(&mut self, vip: &IpAddr) {
        let mut index = 0;
        for connect_vip in &self.current_connect {
            if connect_vip == vip {
                break
            }
            index += 1;
        }
        self.current_connect.remove(index);
    }

}