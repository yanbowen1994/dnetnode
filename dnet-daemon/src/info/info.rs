use std::sync::Mutex;

use dnet_types::team::Team;

use super::ClientInfo;
use super::ProxyInfo;
use super::TincInfo;
use super::error::Result;

static mut EL: *mut Mutex<Info> = 0 as *mut _;

#[derive(Debug, Clone)]
pub struct Info {
    pub client_info:        ClientInfo,
    pub proxy_info:         ProxyInfo,
    pub tinc_info:          TincInfo,
    pub teams:              Vec<Team>,
}

impl Info {
    pub fn new() -> Result<()> {
        let client_info = ClientInfo::new()?;
        let proxy_info = ProxyInfo::new();

        let mut tinc_info = TincInfo::new();
        tinc_info.load_local();

        debug!("client_info: {:?}", client_info);
        debug!("proxy_info: {:?}", proxy_info);
        debug!("tinc_info: {:?}", tinc_info);

        let info = Info {
            client_info,
            proxy_info,
            tinc_info,
            teams: vec![],
        };

        unsafe {
            EL = Box::into_raw(Box::new(Mutex::new(info)));
        }

        Ok(())
    }
}

pub fn get_info() -> &'static Mutex<Info> {
    unsafe {
        if EL == 0 as *mut _ {
            panic!("Get settings instance, before init");
        }
        & *EL
    }
}

pub fn get_mut_info() ->  &'static mut Mutex<Info> {
    unsafe {
        if EL == 0 as *mut _ {
            panic!("Get settings instance, before init");
        }
        &mut *EL
    }
}