use std::thread::spawn;

#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod sys_tool;
pub mod file_tool;
pub mod settings;
pub mod net_tool;
use sys_tool::{cmd, cmd_err_panic, current_platform};
use file_tool::*;
use net_tool::{get_wan_name};
use settings::Settings;

pub fn install_tinc(settings: &Settings) {
    install_on_linux(settings);
}

fn install_on_linux(settings: &Settings) -> i32 {
    let tinc_home  = &settings.tinc.home_path[..];
    let pubkey_path = &settings.tinc.pub_key_path[..];
    cp_tinc_to_local(tinc_home);
    add_dependent("liblzo2-2");
    add_dependent("libcurl4-openssl-dev");
    add_permission_dir(tinc_home);
    config_on_linux(tinc_home);
    create_pub_key(tinc_home, pubkey_path);
    link_ssl(tinc_home);
    if is_tinc_exist(tinc_home) {
        start_tinc(tinc_home);
        if is_tinc_exist(tinc_home) {
            stop_tinc();
            println!("{}", "Success install tinc");
        } else {
            println!("{}", "Failed install tinc");
            return -1;
        }
    } else {
        println!("tinc is running");
    }
//    install_landmark();
    return 0;
}

fn cp_tinc_to_local(tinc_home: &str) {
    cmd_err_panic("rm -rf ".to_string() + tinc_home);
    let path = cur_dir().replace("target/debug", "").replace("target/release", "");
    cmd_err_panic("cp -rf ".to_string() + &path + "/tinc " + tinc_home);
}

fn add_dependent(app_name:&str) {
    if "Ubuntu" == &(current_platform().1) {
        cmd_err_panic("sudo dpkg --get-selections | grep ".to_string() + app_name);

        println!("{}", "**** install Dependency package ".to_string() + app_name);
        cmd_err_panic("sudo apt-get install -y ".to_string() + app_name);
    }
}

fn add_permission_dir(dir_name:&str) {
    let file = File::new(dir_name.to_string());
    if !file.dir_exists() {
        panic!("Dir {} not exists", dir_name);
    };
    cmd_err_panic("chmod 755 ".to_string() + dir_name);
}

fn config_on_linux(tinc_home: &str) {
    let dev = get_wan_name();
    let config;

    let fd = File::new(tinc_home.to_string() + "/bridge.config");

    if fd.file_exists() {
        config = fd.read();
    } else {
        config = dev.unwrap();
    };

    cmd_err_panic("sudo sed -i 's/eth0/".to_string() + &config + "/g' " + tinc_home + "/tinc-up");

    cmd_err_panic("sudo sed -i 's/10.255.255.254/11.255.255.254/g' ".to_string() + tinc_home + "/tinc-up");

    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "/tinc-up");

    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "/tinc");

    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "/tincd");

    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "/start");

//    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "/landmark");
}

fn create_pub_key(tinc_home: &str, pub_key_path: &str) {
    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "key/build-key-tinc.exp");
    cmd_err_panic(tinc_home.to_string()
        + "key/build-key-tinc.exp gen-ed25519-keys "
        + tinc_home
        + "key/rsa_key.priv "
        + tinc_home
        + pub_key_path);
    cmd_err_panic("cp -f ".to_string()
        + tinc_home.clone()
        + "key/rsa_key.p* "
        + tinc_home.clone());
}

fn is_tinc_exist(tinc_home: &str) -> bool {
    check_tinc_status(tinc_home)
}

fn check_tinc_status(tinc_home: &str) -> bool {
    let (_code, output) = cmd(
        "sudo ps aux | grep ".to_string() + tinc_home + "tincd | grep -v 'grep'");

    if output.len() > 0 {
        return true;
    };
    return false;
}

fn start_tinc(tinc_home: &str) {
    let cmd = tinc_home.to_string()
        + "/tincd --config=/root/tinc --pidfile=/root/tinc/tinc.pid &";
    spawn(move ||cmd_err_panic(cmd));
}

fn stop_tinc() {
    cmd("killall tincd &".to_string());
}

fn link_ssl(tinc_home: &str) {
    if !File::new("/usr/lib/libssl.so.1.1".to_string()).file_exists() {
        if File::new("/usr/local/lib/libssl.so.1.1".to_string()).file_exists() {
            cmd_err_panic("sudo ln -s ".to_string()
                + "/usr/local/lib/libssl.so.1.1 "
                + "/usr/lib"
            );
        }
        else {
            cmd_err_panic("sudo ln -s ".to_string()
                + tinc_home
                + "/libssl.so.1.1 "
                + "/usr/lib"
            );
        }
    }

    if !File::new("/usr/lib/libcrypto.so.1.1".to_string()).file_exists() {
        if File::new("/usr/local/lib/libcrypto.so.1.1".to_string()).file_exists() {
            cmd_err_panic("sudo ln -s ".to_string()
                + "/usr/local/lib/libcrypto.so.1.1 "
                + "/usr/lib"
            );
        }
        else {
            cmd_err_panic("sudo ln -s ".to_string()
                + tinc_home
                + "/libcrypto.so.1.1 "
                + "/usr/lib"
            );
        }
    }
}