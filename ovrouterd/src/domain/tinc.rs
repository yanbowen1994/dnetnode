use std::str::FromStr;
use std::net::IpAddr;
use std::fs::File;
use std::io::Read;
use tinc_manager::operator::{self, TincOperator};


pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Can read pubkey file.")]
    PubkeyFile(#[error(cause)] ::std::io::Error),
    #[error(display = "Can read tinc-up file.")]
    GetVip(#[error(cause)] operator::Error),
    #[error(display = "tinc-up vip setting error.")]
    ParseVip(#[error(cause)] ::std::net::AddrParseError),
    #[error(display = "tinc-up vip setting error.")]
    FileNotExit(#[error(cause)] ::std::io::Error),
}

#[derive(Debug, Clone)]
pub struct TincInfo {
    pub vip:     IpAddr,
    pub pub_key: String,
}
impl TincInfo {
    pub fn new() -> Self {
        let vip = IpAddr::from_str("255.255.255.255").unwrap();
        let pub_key = "".to_owned();
        TincInfo {
            vip,
            pub_key,
        }
    }

    // Load local tinc config file vpnserver for tinc vip and pub_key.
    // Success return true.
    pub fn load_local(&mut self, tinc_home: &str, pub_key_path: &str) -> Result<()> {
        if let Ok(vip) = self.load_local_vip(tinc_home) {
            self.vip = vip;
        }
        if let Ok(pub_key) = self.load_local_pubkey(tinc_home, pub_key_path) {
            self.pub_key = pub_key;
        }
        return Ok(());
    }

    fn load_local_vip(&self, tinc_home: &str) -> Result<IpAddr> {
        let tinc = TincOperator::new(tinc_home.to_string());
        IpAddr::from_str(
            &tinc.get_vip().map_err(Error::GetVip)?
        ).map_err(Error::ParseVip)
    }

    fn load_local_pubkey(&self, tinc_home: &str, pub_key_path: &str) -> Result<String> {
        let mut res = String::new();
        let mut _file = File::open(tinc_home.to_string() + pub_key_path)
            .map_err(Error::FileNotExit)?;
        _file.read_to_string(&mut res)
            .map_err(Error::FileNotExit)?;
        Ok(res)
    }
}
