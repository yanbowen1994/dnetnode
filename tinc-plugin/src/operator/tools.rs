#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::str::FromStr;

use std::net::{IpAddr, Ipv4Addr};
use std::io::Read;

use super::{Error, Result};
extern crate openssl;
use openssl::rsa::Rsa;
use sysinfo::{System, SystemExt, ProcessExt};
use crate::operator::TINC_BIN_FILENAME;

pub struct TincTools;

impl TincTools {
    /// 根据IP地址获取文件名
    pub fn get_filename_by_vip(is_proxy: bool, ip: &str) -> String {
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

    pub fn get_vip_by_filename(name: &str) -> Option<IpAddr> {
        let segment: Vec<&str> = name.split("_").collect();
        let mut vip_segment = vec![];

        if segment.len() == 3 {
            for x in segment {
                vip_segment.push(x.parse::<u8>().ok()?);
            }
            Some(IpAddr::from(Ipv4Addr::new(
                10, vip_segment[0], vip_segment[1], vip_segment[2])))
        }
        else if segment.len() == 5 {
            let segment = segment[1..].to_vec();
            for x in segment {
                vip_segment.push(x.parse::<u8>().ok()?);
            }
            Some(IpAddr::from(Ipv4Addr::new(
                10, vip_segment[1], vip_segment[2], vip_segment[3])))
        }
        else {
            None
        }
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
            for _ in 0..5 {
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

    pub fn create_key_pair() -> Result<(String, String)> {
        let key = Rsa::generate(2048)
            .map_err(|_|Error::CreatePubKeyError)?;
        let priv_key = key.private_key_to_pem()
            .map_err(|_|Error::CreatePubKeyError)
            .and_then(|priv_key| String::from_utf8(priv_key)
                .map_err(|_|Error::CreatePubKeyError))?;

        let pubkey = key.public_key_to_pem()
            .map_err(|_|Error::CreatePubKeyError)
            .and_then(|pubkey|String::from_utf8(pubkey)
                .map_err(|_|Error::CreatePubKeyError))?;

        Ok((priv_key, pubkey))
    }

    pub fn get_tinc_pid_file_all_string(path: &str) -> Option<String> {
        if !std::path::Path::new(path).is_file() {
            return None;
        }
        let mut file = std::fs::File::open(path).ok()?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;
        Some(contents)
    }

    pub fn get_tinc_pid_by_sys(
        #[cfg(unix)]
        tinc_home: &str
    ) -> Option<u32> {
        let sys = System::new();
        for (_, info) in sys.get_process_list() {
            if info.name() == TINC_BIN_FILENAME {
                #[cfg(unix)]
                    {
                        let config_buf = "--config=".to_string() + tinc_home;
                        if info.cmd().contains(&config_buf) {
                            return Some(info.pid() as u32);
                        }
                    }
                #[cfg(windows)]
                    return Some(info.pid() as u32);
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use crate::TincTools;

    #[test]
    fn test_get_tinc_pid() {
        let res = TincTools::get_tinc_pid_by_sys("/opt/dnet/tinc/");
        println!("{:?}", res);
    }
}