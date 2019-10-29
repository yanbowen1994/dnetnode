use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::io::Read;
use std::str::FromStr;

use super::{Error, Result};
use super::TincOperator;
use super::{PUB_KEY_FILENAME, TINC_UP_FILENAME};

impl TincOperator {
    /// 从pub_key文件读取pub_key
    pub fn get_local_pub_key(&self) -> Result<String> {
        let _guard = self.mutex.lock().unwrap();
        let path = self.tinc_settings.tinc_home.clone() + PUB_KEY_FILENAME;
        let mut file =  fs::File::open(path.clone())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        Ok(buf)
    }

    /// 获取本地tinc虚拟ip
    pub fn get_local_vip(&self) -> Result<IpAddr> {
        let _guard = self.mutex.lock().unwrap();
        let mut out = String::new();

        let path = self.tinc_settings.tinc_home.clone() + TINC_UP_FILENAME;
        let mut file = fs::File::open(path.clone())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;

        let mut res = String::new();
        file.read_to_string(&mut res)
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        #[cfg(unix)]
            let res: Vec<&str> = res.split("vpngw=").collect();
        #[cfg(windows)]
            let res: Vec<&str> = res.split("addr=").collect();
        if res.len() > 1 {
            let res = res[1].to_string();
            #[cfg(unix)]
                let res: Vec<&str> = res.split("\n").collect();
            #[cfg(windows)]
                let res: Vec<&str> = res.split(" mask").collect();
            if res.len() > 1 {
                out = res[0].to_string();
            }
        }
        Ok(IpAddr::from(Ipv4Addr::from_str(&out).map_err(Error::ParseLocalVipError)?))
    }

    /// 获取子设备公钥
    pub fn get_host_pub_key(&self, host_name: &str) -> Result<String> {
        let _guard = self.mutex.lock().unwrap();
        let file_path = &(self.tinc_settings.tinc_home.to_string() + "hosts/" + host_name);
        let mut file = fs::File::open(file_path)
            .map_err(|_| Error::FileNotExist(file_path.to_string()))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|_| Error::FileNotExist(file_path.to_string()))?;
        Ok(contents)
    }

    pub fn get_tinc_pid(path: &str) -> Result<u64> {
        let mut file = fs::File::open(path)
            .map_err(|_|Error::TincNotExist)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|_|Error::TincNotExist)?;
        let iter: Vec<&str> = contents.split_whitespace().collect();
        if iter.len() < 1 {
            return Err(Error::TincNotExist);
        }
        let pid: u64 = iter[0].parse()
            .map_err(|_|Error::TincNotExist)?;
        Ok(pid)
    }
}