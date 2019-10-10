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

impl DeviceType {
    pub fn get_device_type() -> Self {
        #[cfg(target_arc = "arm")]
            return DeviceType::Router;
        #[cfg(not(target_arc = "arm"))]
            {
                #[cfg(target_os = "linux")]
                    return DeviceType::Linux;
                #[cfg(target_os = "macos")]
                    return DeviceType::MAC;
                #[cfg(target_os = "windows")]
                    return DeviceType::PC;
            }
    }
}