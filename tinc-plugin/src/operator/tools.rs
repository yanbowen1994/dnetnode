use std::net::IpAddr;
use std::str::FromStr;

use super::{Error, Result};

pub struct TincTools;

impl TincTools {
    /// 根据IP地址获取文件名
    pub fn get_filename_by_ip(is_proxy: bool, ip: &str) -> String {
        let splits = ip.split(".").collect::<Vec<&str>>();
        let mut filename = String::new();
        if is_proxy {
            if splits.len() >= 4 {
                filename = "proxy".to_string() + "_";
                filename.push_str(splits[0]);
                filename.push_str("_");
                filename.push_str(splits[1]);
                filename.push_str("_");
                filename.push_str(splits[2]);
                filename.push_str("_");
                filename.push_str(splits[3]);
            }
            else {
                filename.push_str("vpnserver");
            }
        }
        else if splits.len() > 3 {
            filename.push_str(splits[1]);
            filename.push_str("_");
            filename.push_str(splits[2]);
            filename.push_str("_");
            filename.push_str(splits[3]);
        }
        filename
    }


    #[cfg(unix)]
    pub fn set_script_permissions(path: &str) -> Result<()>{
        use std::{os::unix::fs::PermissionsExt};
        std::fs::set_permissions(&path, PermissionsExt::from_mode(0o755))
            .map_err(Error::PermissionsError)?;
        Ok(())
    }

    #[cfg(windows)]
    pub fn get_vnic_index() -> Result<u32> {
        let adapters = ipconfig::get_adapters().unwrap();
        for interface in adapters {
            if interface.friendly_name() == "dnet" {
                return Ok(interface.ipv6_if_index());
            }
        }
        Err(Error::VnicNotFind("No Adapter name \"dnet\" find".to_string()))
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub fn get_default_gateway() -> Result<IpAddr> {
        #[cfg(target_os = "macos")]
        {
            let cmd = duct::cmd!(
                "route", "-n", "get", "default");
            let res = cmd.read().map_err(|e| Error::GetDefaultGatewayError(e.to_string()))?;
            let res: Vec<&str> = res.split("gateway: ").collect();
            if res.len() < 1 {
                return Err(Error::GetDefaultGatewayError("route -n get default not find gateway:".to_string()));
            }
            let res: Vec<&str> = res[1].split("\n").collect();
            let res = res[0];
            IpAddr::from_str(res).map_err(|e| Error::GetDefaultGatewayError(e.to_string()))
        }
        #[cfg(target_os = "windows")]
        {
            let cmd = ::std::process::Command::new("route")
                .args(vec![
                    "print",
                ])
                .output()
                .expect("sh command failed to start");

            let stdout = cmd.stdout;

            let mut res = vec![];
            for i in stdout {
                if i < 32 || 126 < i {
                    continue
                }
                else {
                    res.push(i);
                }
            }

            let mut res = String::from_utf8(res)
                .map_err(|e|Error::GetDefaultGatewayError(e.to_string()))?;
            for i in 0..5 {
                res = res.replace("    ", " ").replace("  ", " ");
            }

            let res: Vec<&str> = res.split("0.0.0.0").collect();
            if res.len() < 2 {
                return Err(Error::GetDefaultGatewayError("route print not find 0.0.0.0 route.".to_string()));
            }

            let res: Vec<&str> = res[2].split(" ").collect();
            let default_gateway_str = res[2];
            let default_gateway = IpAddr::from_str(default_gateway_str)
                .map_err(|e| Error::GetDefaultGatewayError(e.to_string()
                    + "\n" + "route print not find 0.0.0.0 route"))?;
            return Ok(default_gateway);
        }
    }

}