use sys_tool::{cmd_err_panic, cmd};
use file_tool::File;
use super::check::check_tinc_status;

pub struct Operater <'a> {
    tinc_home: &'a str,
    pub_key_path:  &'a str,
}
impl <'a>  Operater <'a>  {
    pub fn new(tinc_home: &'a str, pub_key_path: &'a str) -> Self {
        Operater {
            tinc_home,
            pub_key_path,
        }
    }

    pub fn start_tinc(&self) {
        cmd_err_panic(self.tinc_home.to_string() + "/start");
    }

    pub fn stop_tinc(&self) {
        cmd_err_panic("killall tincd".to_string());
    }

    pub fn is_tinc_exist(&self) -> bool {
        check_tinc_status(self.tinc_home)
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

    pub fn create_pub_key(&self) {
        cmd_err_panic("chmod 755 ".to_string() + self.tinc_home + "key/build-key-tinc.exp");
        cmd_err_panic(self.tinc_home.to_string() + "key/build-key-tinc.exp " + self.tinc_home
            + "key/rsa_key.priv " + self.tinc_home + self.pub_key_path);
    }

    pub fn get_pub_key(&self) -> String {
        let file = File::new(self.tinc_home.to_string() +self.pub_key_path);
        file.read()
    }
}