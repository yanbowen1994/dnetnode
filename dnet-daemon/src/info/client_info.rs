extern crate uuid;

use mac_address::get_mac_address;

use super::error::{Error, Result};
use dnet_types::team::Team;

#[cfg(any(target_arch = "arm", feature = "router_debug"))]
use router_plugin::device_info::DeviceInfo;
#[cfg(any(target_arch = "arm", feature = "router_debug"))]
use crate::settings::get_settings;
#[cfg(target_arch = "arm")]
use router_plugin::get_sn;

use dnet_types::device_type::DeviceType;

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub uid:            String,
    pub cookie:         String,
    pub devicetype:     String,
    pub lan:            String,
    pub wan:            String,
    pub devicename:     String,
    pub running_teams:  Vec<Team>,
    #[cfg(any(target_arch = "arm", feature = "router_debug"))]
    pub device_info:    DeviceInfo,
}
impl ClientInfo {
    pub fn new() -> Result<Self> {
        let device_type = Self::get_device_type();
        let client_info;
        #[cfg(any(target_arch = "arm", feature = "router_debug"))]
            {
                let device_uid = Self::get_uid(&device_type)?;
                let device_info = DeviceInfo::get_info().ok_or(Error::GetDeviceInfo)?;
                client_info = Self {
                    uid:                device_uid.clone(),
                    cookie:             "0cde13b523sf9aa5a403dc9f5661344b91d77609f70952eb488f31641".to_owned(),
                    devicetype:         device_type,
                    lan:                "".to_string(),
                    wan:                "".to_string(),
                    devicename:         device_uid,
                    running_teams:      vec![],
                    device_info,
                }
            }
        #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
            {
                let device_uid = Self::get_uid(&device_type)?;
                client_info = Self {
                    uid:                device_uid.clone(),
                    cookie:             "0cde13b523sf9aa5a403dc9f5661344b91d77609f70952eb488f31641".to_owned(),
                    devicetype:         device_type,
                    lan:                "".to_string(),
                    wan:                "".to_string(),
                    devicename:         device_uid,
                    running_teams:      vec![],
                }
            }
        Ok(client_info)
    }

    fn get_device_type() -> String {
        return format!("{}", (DeviceType::get_device_type() as i8));
    }

    fn get_uid(device_type: &str) -> Result<String> {
        let mac = match get_mac_address() {
            Ok(Some(ma)) => Some(ma.to_string().replace(":", "")),
            Ok(None) => None,
            Err(_) => None,
        }.ok_or(Error::GetMac)?;

        let uid;
        match &device_type[..] {
            #[cfg(feature = "router_debug")]
            "6" => uid = get_settings().common.username.clone(),
            #[cfg(target_arch = "arm")]
            "0" => uid = get_sn().ok_or(Error::GetMac)?,
            #[cfg(not(feature = "router_debug"))]
            "6" => uid = "linux/".to_owned() + &mac,
            "4" => uid = "macos/".to_owned() + &mac,
            "3" => uid = "windows/".to_owned() + &mac,
            _ => uid = "unknown".to_owned() + &mac,
        };
        Ok(uid)
    }
}