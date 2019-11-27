extern crate uuid;

use mac_address::get_mac_address;

use super::error::{Error, Result};

#[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
use router_plugin::device_info::DeviceInfo;
#[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
use crate::settings::get_settings;
#[cfg(target_arch = "arm")]
use router_plugin::get_sn;

use dnet_types::device_type::DeviceType;

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub uid:            String,
    pub devicetype:     DeviceType,
    pub lan:            String,
    pub wan:            String,
    pub devicename:     String,
    #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
    pub device_info:    DeviceInfo,
}
impl ClientInfo {
    pub fn new() -> Result<Self> {
        let device_type = DeviceType::get_device_type();
        let client_info;
        #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
            {
                let device_uid = Self::get_uid(&device_type)?;
                let device_info = DeviceInfo::get_info().ok_or(Error::GetDeviceInfo)?;
                client_info = Self {
                    uid:                device_uid.clone(),
                    devicetype:         device_type,
                    lan:                "".to_string(),
                    wan:                "".to_string(),
                    devicename:         device_uid,
                    device_info,
                }
            }
        #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
            {
                let device_uid = Self::get_uid(&device_type)?;
                client_info = Self {
                    uid:                device_uid.clone(),
                    devicetype:         device_type,
                    lan:                "".to_string(),
                    wan:                "".to_string(),
                    devicename:         device_uid,
                }
            }
        Ok(client_info)
    }

    fn get_uid(device_type: &DeviceType) -> Result<String> {
        let mac = match get_mac_address() {
            Ok(Some(ma)) => Some(ma.to_string().replace(":", "")),
            Ok(None) => None,
            Err(_) => None,
        }.ok_or(Error::GetMac)?;

        let uid;
        match device_type {
            DeviceType::Linux => {
                #[cfg(feature = "router_debug")]
                    {
                        uid = get_settings().common.username.clone();
                    }

                #[cfg(not(feature = "router_debug"))]
                    {
                        uid = "linux/".to_owned() + &mac;
                    }
            }
            #[cfg(target_arch = "arm")]
            DeviceType::Router => uid = get_sn().ok_or(Error::GetMac)?,
            DeviceType::MAC => uid = "macos/".to_owned() + &mac,
            DeviceType::IOS => uid = "ios/".to_owned() + &mac,
            DeviceType::Windows => uid = "windows/".to_owned() + &mac,
            DeviceType::Cloud => uid = "cloud/".to_owned() + &mac,
            _ => uid = "unknown/".to_owned() + &mac,
        };
        Ok(uid)
    }
}