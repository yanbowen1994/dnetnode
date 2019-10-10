extern crate uuid;

use mac_address::get_mac_address;

use super::error::{Error, Result};
use dnet_types::team::Team;

#[cfg(target_arc = "arm")]
use router_plugin::device_info::DeviceInfo;
use dnet_types::device_type::DeviceType;
use crate::settings::get_settings;

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub uid:            String,
    pub cookie:         String,
    pub devicetype:     String,
    pub lan:            String,
    pub wan:            String,
    pub devicename:     String,
    pub running_teams:  Vec<Team>,
    #[cfg(target_arc = "arm")]
    pub device_info:    DeviceInfo,
}
impl ClientInfo {
    pub fn new() -> Result<Self> {
        let device_type = Self::get_device_type();
        let device_uid = Self::get_uid(&device_type)?;
        // TODO
        let device_uid = "4H72675W008AF".to_owned();

        #[cfg(target_arc = "arm")]
            {
                let device_info = DeviceInfo::get_info().ok_or(Error::GetDeviceInfo)?;
            }
        Ok(Self {
            uid:                device_uid.clone(),
            cookie:             "0cde13b523sf9aa5a403dc9f5661344b91d77609f70952eb488f31641".to_owned(),
            devicetype:         device_type,
            lan:                "".to_string(),
            wan:                "".to_string(),
            devicename:         device_uid,
            running_teams:      vec![],
            #[cfg(target_arc = "arm")]
            device_info,
        })
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
            #[cfg(target_arc = "arm")]
            "0" => uid = get_settings().common.username.clone(),
            "6" => uid = "linux/".to_owned() + &mac,
            "4" => uid = "macos/".to_owned() + &mac,
            "3" => uid = "windows/".to_owned() + &mac,
            _ => uid = "unknown".to_owned() + &mac,
        };
        Ok(uid)
    }
}