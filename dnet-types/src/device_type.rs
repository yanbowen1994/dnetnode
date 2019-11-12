#[allow(dead_code)]
#[repr(i8)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum  DeviceType {
    Router                  = 0,
    Android                 = 1,
    IOS                     = 2,
    Windows                 = 3,
    MAC                     = 4,
    Vrouter                 = 5,
    Linux                   = 6,
    Other                   = 7,
}

impl From<u32> for DeviceType {
    fn from(type_code: u32) -> Self {
        match type_code {
            0 => DeviceType::Router,
            1 => DeviceType::Android,
            2 => DeviceType::IOS,
            3 => DeviceType::Windows,
            4 => DeviceType::MAC,
            5 => DeviceType::Vrouter,
            6 => DeviceType::Linux,
            7 => DeviceType::Other,
            _ => DeviceType::Other
        }
    }
}

impl DeviceType {
    pub fn get_device_type() -> Self {
        #[cfg(all(target_os = "linux", any(target_arch = "arm", feature = "router_debug")))]
            return DeviceType::Router;
        #[cfg(not(target_arch = "arm"))]
            {
                #[cfg(target_os = "linux")]
                    return DeviceType::Linux;
                #[cfg(target_os = "macos")]
                    return DeviceType::MAC;
                #[cfg(target_os = "windows")]
                    return DeviceType::Windows;
            }
    }

    pub fn to_string(&self) -> String {
        match self {
            DeviceType::Router    => "Router".to_owned(),
            DeviceType::Android   => "Android".to_owned(),
            DeviceType::IOS       => "IOS".to_owned(),
            DeviceType::Windows   => "Windows".to_owned(),
            DeviceType::MAC       => "MAC".to_owned(),
            DeviceType::Vrouter   => "Vrouter".to_owned(),
            DeviceType::Linux     => "Linux".to_owned(),
            DeviceType::Other     => "Other".to_owned(),
        }
    }
}