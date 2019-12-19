extern crate uuid;

use super::error::{Error, Result};

#[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
use router_plugin::device_info::DeviceInfo;
#[cfg(all(target_os = "linux", feature = "router_debug"))]
use crate::settings::get_settings;
#[cfg(target_arch = "arm")]
use router_plugin::get_sn;
#[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
extern crate base64;

use dnet_types::device_type::DeviceType;
use sandbox::route::{get_default_route, get_mac};

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub devicetype:             DeviceType,
    pub wan:                    String,
    pub device_name:            String,
    #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
    pub device_password:        String,
    #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
    pub device_info:            DeviceInfo,
}
impl ClientInfo {
    pub fn new() -> Result<Self> {
        let device_type = DeviceType::get_device_type();
        let device_name = Self::get_uid(&device_type)?;

        let client_info;
        #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
            {
                let device_password = base64::encode(&device_name);
                let device_info = DeviceInfo::get_info().ok_or(Error::GetDeviceInfo)?;
                client_info = Self {
                    devicetype:          device_type,
                    wan:                 "".to_string(),
                    device_name,
                    device_password,
                    device_info,
                }
            }
        #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
            {
                client_info = Self {
                    devicetype:         device_type,
                    wan:                "".to_string(),
                    device_name,
                }
            }
        Ok(client_info)
    }

    fn get_uid(device_type: &DeviceType) -> Result<String> {
        let mac = get_default_route()
            .and_then(|route_info| {
                get_mac(&route_info.dev)
            })
            .ok_or(Error::GetMac)?;

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

    pub fn get_lan_str(&self) -> String {
        // route mode
        #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
            {
                serde_json::to_string(&self.device_info.lan)
                    .map_err(|e| {
                        error!("JavaDevice::new() parse lan failed. {:?}", e);
                    })
                    .unwrap_or("".to_string())
            }
        // x86
        #[cfg(all(not(target_arch = "arm"), not(feature = "router_debug")))]
            {
                "".to_owned()
            }
    }
}

#[cfg(test)]
mod test {
    use mac_address::get_mac_address;

    #[test]
    fn test_get_mac_address() {
        let mac = get_mac_address().unwrap().unwrap().to_string();
        println!("{:?}", mac);
    }
}