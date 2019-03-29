use std::path;
use std::fs;
use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use sys_tool::{cmd_err_panic};
use file_tool::File;
use net_tool::get_wan_name;
use super::check::check_tinc_status;
use domain::Info;
use core::borrow::Borrow;
use domain::AuthInfo;

const TINC_AUTH_PATH: &str = "auth/";
const TINC_AUTH_FILENAME: &str = "auth.txt";

pub struct Tinc {
    tinc_home:          String,
    pub_key_path:       String,
    tinc_handle:         Option<duct::Handle>,
}
impl Tinc {
    pub fn new(tinc_home: String, pub_key_path: String) -> Self {
        Tinc {
            tinc_home,
            pub_key_path,
            tinc_handle: None,
        }
    }

    pub fn start_tinc(&mut self) -> bool {
        let argument: Vec<&str> = vec![];
        let tinc_handle: duct::Expression = duct::cmd(
            OsString::from(self.tinc_home.to_string() + "/tincd"),
            argument).unchecked();
        if let Ok(child) = tinc_handle.start() {
            self.tinc_handle = Some(child);
            return true;
        }
        return false;
    }

    pub fn stop_tinc(&mut self) -> bool {
        if let Some(child) = &self.tinc_handle {
            if let Ok(_) = child.kill() {
                ()
            }
        }
        self.tinc_handle = None;
        return true;
    }

    pub fn is_tinc_exist(&self) -> bool {
        check_tinc_status(&self.tinc_home)
    }

    pub fn restart_tinc(&mut self) {
        if self.is_tinc_exist() {
            self.stop_tinc();
        }
        for i in 0..10 {
            self.start_tinc();
            if !self.is_tinc_exist() {
                if i == 9 {
                    panic!("Error: Fail to restart tinc.");
                }
            } else {
                break;
            }
        }
    }

    /// 根据IP地址获取文件名
    pub fn get_filename_by_ip(&self, ip: &str) -> String{
        let splits = ip.split(".").collect::<Vec<&str>>();
        let mut filename = String::new();
        filename.push_str(splits[0]);
        filename.push_str("_");
        filename.push_str(splits[1]);
        filename.push_str("_");
        filename.push_str(splits[2]);
        filename.push_str("_");
        filename.push_str(splits[3]);
        filename
    }

    /// 根据IP地址获取文件名
    pub fn get_client_filename_by_virtual_ip(&self, virtual_ip: &str) -> String{
        let splits = virtual_ip.split(".").collect::<Vec<&str>>();
        let mut filename = String::new();
        filename.push_str(splits[1]);
        filename.push_str("_");
        filename.push_str(splits[2]);
        filename.push_str("_");
        filename.push_str(splits[3]);
        filename
    }

    /// 添加子设备
    pub fn add_hosts(&self, host_name: &str, pub_key: &str) -> bool {
        let file:File = File::new(format!("{}/{}/{}",self.tinc_home.clone() , "hosts",host_name));
        file.write(pub_key.to_string())
    }

    /// 获取子设备公钥
    pub fn get_host_pub_key(&self,host_name:&str) -> String{
        debug!("get_host_pub_key: {}",format!("{}/{}/{}",self.tinc_home.clone() , "hosts",host_name));
        let file = File::new(format!("{}/{}/{}",self.tinc_home.clone() , "hosts",host_name));
        file.read()
    }

    pub fn create_pub_key(&self) {
        cmd_err_panic("chmod 755 ".to_string() + &self.tinc_home + "key/build-key-tinc.exp");
        cmd_err_panic(self.tinc_home.clone() + "key/build-key-tinc.exp gen-ed25519-keys " + &self.tinc_home + "key/rsa_key.priv " + &self.tinc_home + &self.pub_key_path);
        cmd_err_panic("cp -f ".to_string()
                          + &self.tinc_home.clone()
                          + "key/rsa_key.p* "
                          + &self.tinc_home.clone());
    }

    /// 从pub_key文件读取pub_key
    pub fn get_pub_key(&self) -> String {
        let file = File::new(self.tinc_home.clone() + &self.pub_key_path);
        file.read()
    }

    pub fn set_pub_key(&mut self, pub_key: &str) -> bool {
        let file = File::new(self.tinc_home.clone() + &self.pub_key_path);
        file.write(pub_key.to_string())
    }

    pub fn get_vip(&self) -> String {
        let mut out = String::new();

        let file = File::new(self.tinc_home.clone() + "tinc-up");
        let res = file.read();
        let res: Vec<&str> = res.split("vpngw=").collect();
        if res.len() > 1 {
            let res = res[1].to_string();
            let res: Vec<&str> = res.split("\n").collect();
            if res.len() > 1 {
                out = res[0].to_string();
            }
        }
        return out;
    }

    fn set_tinc_conf_file(&self, info: &Info) -> bool {
        let name = "proxy".to_string() + "_"
            + &self.get_filename_by_ip(&info.proxy_info.proxy_ip);

        let mut connect_to: Vec<String> = vec![];
        for online_proxy in info.proxy_info.online_porxy.clone() {
            let online_proxy_name = "proxy".to_string() + "_"
                + &self.get_filename_by_ip(&online_proxy.ip.to_string());
            connect_to.push(online_proxy_name);
        }


        let mut buf_connect_to = String::new();
        for other in connect_to {
            let buf = "ConnectTo = ".to_string() + &other + "\n\
            ";
            buf_connect_to += &buf;
        }
        let buf = "Name = ".to_string() + &name + "\n\
        " + &buf_connect_to
        + "DeviceType=tap\n\
        Mode=switch\n\
        Interface=tun0\n\
        Device = /dev/net/tun\n\
        BindToAddress = * 50069\n\
        ProcessPriority = high\n\
        PingTimeout=10";
        let file = File::new(self.tinc_home.clone() + "/tinc.conf");
        file.write(buf.to_string())
    }

    /// 检查info中的配置, 并与实际运行的tinc配置对比, 如果不同修改tinc配置,
    /// 如果自己的vip修改,重启tinc
    pub fn check_info(&mut self, info: &Info) -> bool {
        let mut need_restart = false;
        {
            let file_vip = self.get_vip();
            if file_vip != info.tinc_info.vip.to_string() {
                debug!("tinc operator check_info local {}, remote {}",
                       file_vip,
                       info.tinc_info.vip.to_string());

                if !self.change_vip(info.tinc_info.vip.to_string()) {
                    return false;
                }

                if !self.set_hosts(true,
                                   &info.proxy_info.proxy_ip.to_string(),
                                   &info.tinc_info.pub_key,
                ) {
                    return false;
                }

                need_restart = true;
            }
        }
        {
            for online_proxy in info.proxy_info.online_porxy.clone() {
                if !self.set_hosts(true,
                                   &online_proxy.ip.to_string(),
                                   &online_proxy.pubkey,
                ) {
                    return false;
                }
            }
        }

        if self.check_self_hosts_file(self.tinc_home.borrow(), &info) {
            self.set_hosts(
                true,
                &info.proxy_info.proxy_ip,
                &info.tinc_info.pub_key);
        }

        if need_restart {
            self.set_tinc_conf_file(&info);
            self.restart_tinc();
        }
        return true;
    }

    fn set_hosts(&self,
                     is_proxy: bool,
                     ip: &str,
                     pubkey: &str) -> bool {
        {
            let mut proxy_or_client = "proxy".to_string();
            if !is_proxy {
                proxy_or_client = "CLIENT".to_string();
            }
            let buf = "Address=".to_string()
                + ip
                + "\n\
                "
                + pubkey
                + "Port=50069\n\
                ";
            let file_name = proxy_or_client.to_string()
                + "_" + &self.get_filename_by_ip(ip);
            let file = File::new(self.tinc_home.clone() + "/hosts/" + &file_name);
            if !file.write(buf.to_string()) {
                return false;
            }
        }
        true
    }

    /// 修改tinc虚拟ip
    fn change_vip(&self, vip: String) -> bool {
        let wan_name = match get_wan_name() {
            Some(x) => x,
            None => {
                warn!("change_vip get dev wan failed, use defualt.");
                "eth0".to_string()
            }
        };
        {
            let buf = "#! /bin/sh\n\
            dev=tun0\n\
            vpngw=".to_string() + &vip + "\n\
            echo 1 > /proc/sys/net/ipv4/ip_forward\n\
            ifconfig ${dev} ${vpngw} netmask 255.0.0.0\n\
            iptables -t nat -F\n\
            iptables -t nat -A POSTROUTING -s ${vpngw}/8 -o "
            + &wan_name
            + " -j MASQUERADE\n\
            exit 0";
            let file = File::new(self.tinc_home.clone() + "/tinc-up");
            if !file.write(buf.to_string()) {
                return false;
            }
        }
        true
    }

    pub fn check_self_hosts_file(&self, tinc_home: &str, info: &Info) -> bool {
        let ip = info.proxy_info.proxy_ip.clone();
        let filename = self.get_filename_by_ip(&ip);
        let file = File::new(
            tinc_home.to_string()
                + "/hosts/"
                + "proxy_"
                + &filename
        );
        file.file_exists()
    }

    pub fn write_auth_file(&self,
                           server_url:  &str,
                           info:        &Info,
    ) -> bool {
        let auth_dir = path::PathBuf::from(&(self.tinc_home.to_string() + TINC_AUTH_PATH));
        if !path::Path::new(&auth_dir).is_dir() {
            if let Err(_) = fs::create_dir_all(&auth_dir) {
                return false;
            }
        }

        let file_path_buf = auth_dir.join(TINC_AUTH_FILENAME);
        let file_path = path::Path::new(&file_path_buf);

        let permissions = PermissionsExt::from_mode(0o755);
        if file_path.is_file() {
            if let Ok(file) = fs::File::open(&file_path) {
                if let Err(_) = file.set_permissions(permissions) {
                    return false;
                }
            }
            else {
                return false;
            }
        }
        else {
            if let Ok(file) = fs::File::create(&file_path) {
                if let Err(_) = file.set_permissions(permissions) {
                    return false;
                }
            }
            else {
                return false;
            }
        }
        if let Some(file_str) = file_path.to_str() {
            let file = File::new(file_str.to_string());
            let auth_info = AuthInfo::load(server_url, info);
            file.write(auth_info.to_json_str());
        }
        else {
            return false;
        }
        return true;
    }
}