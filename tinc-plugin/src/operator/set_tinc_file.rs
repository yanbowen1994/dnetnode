use std::fs;
use std::path;
use std::io::Write;
use std::time::SystemTime;

extern crate openssl;
use openssl::rsa::Rsa;

use crate::info::{TincRunMode, TincInfo};
use super::{Error, Result, TincOperator,
            PUB_KEY_FILENAME, TINC_UP_FILENAME, TINC_DOWN_FILENAME,
            HOST_UP_FILENAME, HOST_DOWN_FILENAME, PRIV_KEY_FILENAME};
use crate::operator::TincTools;

impl TincOperator {
    pub fn set_info_to_local(&mut self, info: &TincInfo) -> Result<()> {
        self.set_tinc_conf_file(info)?;
        let is_proxy = match self.tinc_settings.mode {
            TincRunMode::Proxy => true,
            TincRunMode::Client => false,
        };

        self.set_tinc_up(&info)?;
        self.set_tinc_down(info)?;
        if is_proxy {
            self.set_host_up()?;
            self.set_host_down()?;
        }

        for online_proxy in info.connect_to.clone() {
            self.set_hosts(true,
                           &online_proxy.ip.to_string(),
                           &online_proxy.pubkey)?;
        };

        let ip;
        if is_proxy {
            ip = info.ip
                .ok_or(Error::TincInfoProxyIpNotFound)?
                .to_string();
        }
        else {
            ip = info.vip.to_string();
        }
        self.set_hosts(is_proxy, &ip, &info.pub_key)
    }

    fn set_tinc_up(&self, tinc_info: &TincInfo) -> Result<()> {
        let _guard = self.mutex.lock().unwrap();

        let netmask = match self.tinc_settings.mode {
            TincRunMode::Proxy => "255.0.0.0",
            TincRunMode::Client => "255.255.255.255",
        };

        let mut buf;

        #[cfg(target_arch = "arm")]
            {
                buf = "#!/bin/sh\n\
                dev=dnet\n\
                vpngw=".to_string() + &tinc_info.vip.to_string() + "\n" +
                    "ifconfig ${dev} up\n\
                     ifconfig ${dev} ${vpngw} netmask " + netmask;

                buf = buf + "\n" + &self.tinc_settings.tinc_home + "tinc-report -u";
            }
        #[cfg(all(target_os = "linux", not(target_arch = "arm")))]
            {
                buf = "#!/bin/bash\n\
            dev=dnet\n\
            vpngw=".to_string() + &tinc_info.vip.to_string() + "\n" +
                    "ifconfig ${dev} ${vpngw} netmask " + netmask;

//          Example for global proxy
//
// ```
//            if TincRunMode::Client == self.tinc_settings.mode {
//                if tinc_info.connect_to.is_empty() {
//                    return Err(Error::TincInfo_connect_to_is_empty)
//                }
//
//                buf = buf + "\n"
//                    + "route add -host " + &tinc_info.connect_to[0].ip.to_string() + " gw _gateway";
//                buf = buf + "\n"
//                    + "route add -host 10.255.255.254 dev dnet";
//                buf = buf + "\n"
//                    + "route add default gw " + &tinc_info.connect_to[0].vip.to_string();
//            }
// ```

                buf = buf + "\n" + &self.tinc_settings.tinc_home + "tinc-report -u";
            }
        #[cfg(target_os = "macos")]
            {
                buf = "#!/bin/bash\n\
                   dev=tap0\n\
                   vpngw=".to_string()
                    + &tinc_info.vip.to_string() + "\n"
                    + "ifconfig ${dev} ${vpngw} netmask " + netmask + "\n";

// Example for global proxy
// ```
//  if TincRunMode::Client == self.tinc_settings.mode {
//      let default_gateway = get_default_gateway()?.to_string();
//      buf = buf
//          + "route -q -n delete -net 0.0.0.0\n\
//      route -q -n add -host " + &tinc_info.connect_to[0].ip.to_string()
//          + " -gateway " + &default_gateway + "\n"
//          + "route add -host 10.255.255.254 -interface tap0 -iface -cloning\n"
//          + "route add -net 0.0.0.0 -gateway 10.255.255.254";
//  }
// ```

                buf = buf + &self.tinc_settings.tinc_home + "tinc-report -u\n";
            }
        #[cfg(windows)]
            {
                buf = "netsh interface ipv4 set address name=\"dnet\" source=static addr=".to_string() +
                    &tinc_info.vip.to_string() + " mask=" + netmask + "\r\n";

//          Example for global proxy
//            if TincRunMode::Client == self.tinc_settings.mode {
//                let default_gateway = get_default_gateway()?.to_string();
//                let vnic_index = format!("{}", get_vnic_index()?);
//
//                buf = buf
//                    + "route add " + &tinc_info.connect_to[0].ip.to_string()
//                        + " mask 255.255.255.255 " + &default_gateway + "\r\n"
//                    + "route add 10.255.255.254 mask 255.255.255.255 10.255.255.254 if "
//                        + &vnic_index + "\r\n"
//                    + "route add 0.0.0.0 mask 0.0.0.0 10.255.255.254 if "
//                        + &vnic_index + "\r\n";
//            }
                buf = buf + &self.tinc_settings.tinc_home + "tinc-report.exe -u";
            }

        let path = self.tinc_settings.tinc_home.clone() + TINC_UP_FILENAME;
        let mut file = fs::File::create(path.clone())
            .map_err(|e|Error::FileCreateError(path.clone() + " " + &e.to_string()))?;
        file.write(buf.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        #[cfg(unix)]
            TincTools::set_script_permissions(&path)?;
        Ok(())
    }

    fn set_tinc_down(&self, _tinc_info: &TincInfo) -> Result<()> {
        let _guard = self.mutex.lock().unwrap();
        let buf;
        #[cfg(target_arch = "arm")]
            {
                buf = "#!/bin/sh\n".to_string() + &self.tinc_settings.tinc_home + "tinc-report -d";
            }
        #[cfg(all(target_os = "linux", not(target_arch = "arm")))]
            {
                buf = "#!/bin/bash\n".to_string() + &self.tinc_settings.tinc_home + "tinc-report -d";
            }
        #[cfg(target_os = "macos")]
            {
                let default_gateway = get_default_gateway()?.to_string();
                buf = "#!/bin/bash\n".to_string() + &self.tinc_settings.tinc_home + "tinc-report -d";

//              Example for global proxy
//                    + "route -n -q delete -host " + &tinc_info.connect_to[0].ip.to_string() + "\n"
//                    + "route -n -q delete -net 0.0.0.0 \n\
//                       route -n -q add -net 0.0.0.0 -gateway " + &default_gateway;
            }
        #[cfg(windows)]
            {
                let vnic_index = format!("{}", TincTools::get_vnic_index()?);
                buf = self.tinc_settings.tinc_home.to_string() + "tinc-report.exe -d";
//              Example for global proxy
//                buf = "route delete 0.0.0.0 mask 0.0.0.0 10.255.255.254 if ".to_string()
//                    + &vnic_index + "\r\n"
//                    + &self.tinc_settings.tinc_home.to_string() + "tinc-report.exe -d";
            }

        let path = self.tinc_settings.tinc_home.clone() + TINC_DOWN_FILENAME;
        let mut file = fs::File::create(path.clone())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        file.write(buf.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        #[cfg(unix)]
             TincTools::set_script_permissions(&path)?;
        Ok(())
    }

    fn set_host_up(&self) -> Result<()> {
        let _guard = self.mutex.lock().unwrap();
        #[cfg(windows)]
            let buf = &(self.tinc_settings.tinc_home.to_string() + "tinc-report.exe -hu ${NODE}");
        #[cfg(unix)]
            let buf = "#!/bin/bash\n".to_string() + &self.tinc_settings.tinc_home + "tinc-report -hu ${NODE}";

        let path = self.tinc_settings.tinc_home.clone() + HOST_UP_FILENAME;
        let mut file = fs::File::create(path.clone())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        file.write(buf.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        #[cfg(unix)]
            TincTools::set_script_permissions(&path)?;
        Ok(())
    }

    fn set_host_down(&self) -> Result<()> {
        let _guard = self.mutex.lock().unwrap();
        #[cfg(windows)]
            let buf = &(self.tinc_settings.tinc_home.to_string() + "tinc-report.exe -hd ${NODE}");
        #[cfg(unix)]
            let buf = "#!/bin/bash\n".to_string() + &self.tinc_settings.tinc_home + "tinc-report -hd ${NODE}";

        let path = self.tinc_settings.tinc_home.clone() + HOST_DOWN_FILENAME;
        let mut file = fs::File::create(path.clone())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        file.write(buf.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        #[cfg(unix)]
            TincTools::set_script_permissions(&path)?;
        Ok(())
    }

    pub fn create_tinc_dirs(&self) -> Result<()> {
        let path_str = self.tinc_settings.tinc_home.clone() + "hosts";
        if !std::path::Path::new(&path_str).is_dir() {
            fs::create_dir_all(&path_str)
                .map_err(|_| Error::IoError("Can not create tinc home dir".to_string()))?;
        }
        Ok(())
    }


    pub fn check_pub_key(&self) -> bool {
        let pubkey_path = self.tinc_settings.tinc_home.to_owned() + PUB_KEY_FILENAME;
        let path = path::Path::new(&pubkey_path);
        if let Ok(fs) = std::fs::metadata(path) {
            if let Ok(time) = fs.modified() {
                if let Ok(now) = SystemTime::now().duration_since(time) {
                    if now.as_secs() / 60 / 60 / 24 < 30 {
                        return true;
                    }
                }
            }
        }
        return false;
    }

    /// openssl Rsa 创建2048位密钥对, 并存放到tinc配置文件中
    pub fn create_pub_key(&self) -> Result<()> {
        let _guard = self.mutex.lock().unwrap();
        let mut write_priv_key_ok = false;
        if let Ok(key) = Rsa::generate(2048) {
            if let Ok(priv_key) = key.private_key_to_pem() {
                if let Ok(priv_key) = String::from_utf8(priv_key) {
                    let mut file = fs::File::create(
                        self.tinc_settings.tinc_home.to_string() + PRIV_KEY_FILENAME)
                        .map_err(|e|
                            Error::FileCreateError((self.tinc_settings.tinc_home.to_string() + PRIV_KEY_FILENAME)
                                + " " + &e.to_string()))?;
                    file.write_all(priv_key.as_bytes())
                        .map_err(|_|Error::CreatePubKeyError)?;
                    drop(file);

                    write_priv_key_ok = true;
                }
            }
            if let Ok(pub_key) = key.public_key_to_pem() {
                if let Ok(pub_key) = String::from_utf8(pub_key) {
                    let path = self.tinc_settings.tinc_home.to_string() + PUB_KEY_FILENAME;
                    let mut file = fs::File::create(&path)
                        .map_err(|e|Error::FileCreateError(path.clone() + " " + &e.to_string()))?;
                    file.write_all(pub_key.as_bytes())
                        .map_err(|_|Error::CreatePubKeyError)?;
                    drop(file);
                    if write_priv_key_ok {
                        return Ok(());
                    }
                }
            }
        }
        Err(Error::CreatePubKeyError)
    }

    /// 修改本地公钥
    pub fn set_local_pub_key(&self, pub_key: &str) -> Result<()> {
        let _guard = self.mutex.lock().unwrap();
        let path = self.tinc_settings.tinc_home.clone() + PUB_KEY_FILENAME;
        let mut file =  fs::File::create(path.clone())
            .map_err(|_|Error::CreatePubKeyError)?;
        file.write(pub_key.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        return Ok(());
    }
    /// 通过Info修改tinc.conf
    fn set_tinc_conf_file(&self, tinc_info: &TincInfo) -> Result<()> {
        let _guard = self.mutex.lock().unwrap();

        let (is_proxy, name_ip) = match self.tinc_settings.mode {
            TincRunMode::Proxy => {
                let name_ip = tinc_info.ip.clone()
                    .ok_or(Error::TincInfoProxyIpNotFound)?;
                (true, name_ip)
            },
            TincRunMode::Client => (false, tinc_info.vip.clone()),
        };

        let name = TincTools::get_filename_by_ip(is_proxy,
                                            &name_ip.to_string());

        let mut connect_to: Vec<String> = vec![];
        for online_proxy in tinc_info.connect_to.clone() {
            let online_proxy_name = TincTools::get_filename_by_ip(true,
                                                             &online_proxy.ip.to_string());
            connect_to.push(online_proxy_name);
        }


        let mut buf_connect_to = String::new();
        for other in connect_to {
            let buf = "ConnectTo = ".to_string() + &other + "\n";
            buf_connect_to += &buf;
        }

        let buf;
        #[cfg(target_os = "linux")]
            {
                buf = "Name = ".to_string() + &name + "\n"
                    + &buf_connect_to
                    + "DeviceType=tap\n\
                   Mode=switch\n\
                   Interface=dnet\n\
                   BindToAddress = * 50069\n\
                   ProcessPriority = high\n\
                   PingTimeout=3\n\
                   Device = /dev/net/tun\n\
                   AutoConnect=no\n\
                   MaxConnectionBurst=1000\n";
            }
        #[cfg(target_os = "macos")]
            {
                buf = "Name = ".to_string() + &name + "\n"
                    + &buf_connect_to
                    + "DeviceType=tap\n\
                   Mode=switch\n\
                   Interface=dnet\n\
                   BindToAddress = * 50069\n\
                   ProcessPriority = high\n\
                   PingTimeout=3\n\
                   Device = /dev/tap0\n\
                   AutoConnect=no\n\
                   MaxConnectionBurst=1000\n";
            }
        #[cfg(windows)]
            {
                buf = "Name = ".to_string() + &name + "\n"
                    + &buf_connect_to
                    + "DeviceType=tap\n\
                   Mode=switch\n\
                   Interface=dnet\n\
                   BindToAddress = * 50069\n\
                   ProcessPriority = high\n\
                   PingTimeout=3\n\
                   AutoConnect=no\n\
                   MaxConnectionBurst=1000\n";
            }

        let path = self.tinc_settings.tinc_home.clone() + "tinc.conf";
        let mut file = fs::File::create(path.clone())
            .map_err(|e|Error::FileCreateError(path.clone() + " " + &e.to_string()))?;
        file.write(buf.as_bytes())
            .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        return Ok(());
    }

    /// 添加hosts文件
    /// if is_proxy{ 文件名=proxy_10_253_x_x }
    /// else { 文件名=虚拟ip后三位b_c_d }
    pub fn set_hosts(&self,
                     is_proxy: bool,
                     ip: &str,
                     pubkey: &str)
                     -> Result<()> {
        let _guard = self.mutex.lock().unwrap();
        {
            let buf;
            if is_proxy {
                buf = "Address=".to_string() + ip + "\n"
                    + "Port=50069\n"
                    + pubkey;
            }
            else {
                buf = pubkey.to_string();
            }
            let file_name = TincTools::get_filename_by_ip(is_proxy, ip);

            let path = self.tinc_settings.tinc_home.clone() + "hosts/" + &file_name;
            let mut file = fs::File::create(path.clone())
                .map_err(|e|Error::FileCreateError(path.clone() + " " + &e.to_string()))?;
            file.write(buf.as_bytes())
                .map_err(|e|Error::IoError(path.clone() + " " + &e.to_string()))?;
        }
        Ok(())
    }

}