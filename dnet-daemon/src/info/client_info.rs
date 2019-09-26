extern crate uuid;

use mac_address::get_mac_address;

use super::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub uid:        String,
    pub cookie:     String,
    pub devicetype: String,
    pub lan:        String,
    pub wan:        String,
    pub devicename: String,
}
impl ClientInfo {
    pub fn new() -> Result<Self> {
        let device_type = Self::get_device_type();
        let device_uid = Self::get_uid(&device_type)?;
        Ok(Self {
            uid:                device_uid.clone(),
            cookie:             "0cde13b523sf9aa5a403dc9f5661344b91d77609f70952eb488f31641".to_owned(),
            devicetype:         device_type,
            lan:                "".to_string(),
            wan:                "".to_string(),
            devicename:         device_uid,
        })
    }

    fn get_device_type() -> String {
        #[cfg(target_arc = "arm")]
            return (DeviceType::Router as i8) as String;
        #[cfg(not(target_arc = "arm"))]
            {
                #[cfg(target_os = "linux")]
                    return format!("{}", (DeviceType::Linux as i8));
                #[cfg(target_os = "macos")]
                    return format!("{}", (DeviceType::MAC as i8));
                #[cfg(target_os = "windows")]
                    return format!("{}", (DeviceType::PC as i8));
            }
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
            "0" => uid = router_plugin::get_sn(),
            "6" => uid = "linux/".to_owned() + &mac,
            "4" => uid = "macos/".to_owned() + &mac,
            "3" => uid = "windows/".to_owned() + &mac,
            _ => uid = "unknown".to_owned() + &mac,
        };
        Ok(uid)
    }
}

#[allow(dead_code)]
#[repr(i8)]
pub enum  DeviceType {
    Router                  = 0,
    Android                 = 1,
    IOS                     = 2,
    PC                      = 3,
    MAC                     = 4,
    Vrouter                 = 5,
    Linux                   = 6,
    Other                   = 7,
}