use sys_tool::{cmd_err_panic, cmd};
use file_tool::File;
use super::check::check_tinc_status;

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
        cmd_err_panic(self.tinc_home.clone() + "/start");
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

    //! 添加子设备
    pub fn add_hosts(&self, hosts_name: &str, vip: &str) -> bool {
        let file = File::new(self.tinc_home.clone() + "/hosts/" + hosts_name);
        file.write(vip.to_string())
    }

    pub fn create_pub_key(&self) {
        cmd_err_panic("chmod 755 ".to_string() + &self.tinc_home + "key/build-key-tinc.exp");
        cmd_err_panic(self.tinc_home.clone() + "key/build-key-tinc.exp " + &self.tinc_home
            + "key/rsa_key.priv " + &self.tinc_home + &self.pub_key_path);
    }

    //! 从pub_key文件读取pub_key
    pub fn get_pub_key(&self) -> String {
        let file = File::new(self.tinc_home.clone() + &self.pub_key_path);
        file.read()
    }

    pub fn set_pub_key(&mut self, pub_key: &str) -> bool {
        let file = File::new(self.tinc_home.clone() + &self.pub_key_path);
        file.write(pub_key.to_string())
    }

    //! 修改tinc虚拟ip
    pub fn change_vip(&self, vip: String) -> bool {
        {
            let buf = "Address = ".to_string()
                + &vip
                + "\nSubnet = "
                + &vip
                + "/32\nPrivateKeyFile = "
                + &self.tinc_home
                + &self.pub_key_path;

            let file = File::new(self.tinc_home.clone() + "/hosts/vpnserver");
            if !file.write(buf.to_string()) {
                return false;
            }
        }
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
}