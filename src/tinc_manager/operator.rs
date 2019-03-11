use sys_tool::{cmd_err_panic, cmd};
use file_tool::File;
use super::check::check_tinc_status;
use domain::{Info, TincInfo, ProxyInfo, OnlineProxy};

pub struct Tinc {
    tinc_home: String,
    pub_key_path:  String,
}
impl Tinc {
    pub fn new(tinc_home: String, pub_key_path: String) -> Self {
        Tinc {
            tinc_home,
            pub_key_path,
        }
    }

    pub fn start_tinc(&self) {
        cmd_err_panic(self.tinc_home.clone() + "/start &");
    }

    pub fn stop_tinc(&self) {
        cmd("killall tincd".to_string());
    }

    pub fn is_tinc_exist(&self) -> bool {
        check_tinc_status(&self.tinc_home)
    }

    pub fn restart_tinc(&self) {
        for i in 0..3 {
            if self.is_tinc_exist() {
                self.stop_tinc();
            }
            self.start_tinc();
            if !self.is_tinc_exist() {
                if i == 2 {
                    panic!("Error: Fail to restart tinc.");
                }
            } else {
                break;
            }
        }
    }

    /// 根据IP地址获取文件名
    pub fn get_filename_by_virtual_ip(&self,virtual_ip:&str) -> String{
        let splits = virtual_ip.split(".").collect::<Vec<&str>>();
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
        cmd_err_panic(self.tinc_home.clone() + "key/build-key-tinc.exp " + &self.tinc_home + "key/rsa_key.priv " + &self.tinc_home + &self.pub_key_path);
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
        let file = File::new(self.tinc_home.clone() + "tinc-up");
        let res = file.read();
        let res = res.split("vpngw=").collect();
        if res.len() > 1 {
            let res = res[1];
        }
        let res = res.split("\n").collect();
        if res.len() > 1 {
            let res = res[0];
        }
        return res;
    }

    pub fn set_hosts(&self,
                     proxy_or_client: &str,
                     ip: &str,
                     vip: &str,
                     pubkey: &str) -> bool {
        {
            let buf = "Address = ".to_string()
                + vip
                + "\nSubnet = "
                + vip
                + "/32\n"
                + pubkey;
            let vip_name_vec: Vec<&str> = vip.split(".").collect();

            let file_name = proxy_or_client.to_string()
                + "_" + vip_name_vec[1]
                + "_" + vip_name_vec[2]
                + "_" + vip_name_vec[3];
            let file = File::new(self.tinc_home.clone() + "/hosts/" + &file_name);
            if !file.write(buf.to_string()) {
                return false;
            }
        }
        true
    }

    pub fn clean_host_online_proxy(&self) -> bool {
        let hosts_dir = self.tinc_home.to_string() + "/hosts";
        let paths = File::new(hosts_dir.clone());
        let files = paths.ls();
        for file in files {

        }
        true
    }

    /// 修改tinc虚拟ip
    pub fn change_vip(&self, vip: String) -> bool {
        {
            let buf = "#! /bin/sh\n
            dev=tun0\n
            vpngw=".to_string() + &vip + "\n
            echo 1 > /proc/sys/net/ipv4/ip_forward\n
            ifconfig ${dev} ${vpngw} netmask 255.0.0.0\n
            iptables -t nat -F\n
            iptables -t nat -A POSTROUTING -s ${vpngw}/8 -o enp0s3 -j MASQUERADE\nexit 0";
            let file = File::new(self.tinc_home.clone() + "/tinc-up");
            if !file.write(buf.to_string()) {
                return false;
            }
        }
        true
    }

    pub fn check_info(&self, info: &Info) {
        {
            let file_vip = self.get_vip();
            if file_vip != info.tinc_info.vip.to_string() {
                self.change_vip(info.tinc_info.vip.to_string());
            }
        }
    }
}