use file_tool::*;
use sys_tool::*;
use net_tool::{get_wan_name};
use super::operate::*;
use settings::Settings;

pub fn install_tinc(settings: &Settings) {
    install_on_linux(settings);
}

fn install_on_linux(settings: &Settings) -> i32 {
    let tinc_home  = &settings.tinc.home_path[..];
    let pub_key_path = &settings.tinc.pub_key_path[..];
    cp_tinc_to_local(tinc_home);
    add_dependent("liblzo2-2");
    add_permission_dir(tinc_home);
    config_on_linux(tinc_home);
    create_pub_key(tinc_home, pub_key_path);
    if !is_tinc_exist(tinc_home) {
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

    cmd_err_panic("cp -rf ./tinc ".to_string() + tinc_home);
}

fn add_dependent(app_name:&str) {
    if "Ubuntu" == &(current_platform().1) {
        cmd_err_panic("sudo dpkg --get-selections | grep ".to_string() + app_name);

        println!("{}", "**** install Dependency package ".to_string() + app_name);
        cmd_err_panic("sudo apt-get install -y ".to_string() + app_name);
    }
}

fn add_permission_dir(dir_name:&str) {
    if !dir_exists(&dir_name.to_string()) {
        panic!("Dir {} not exists", dir_name);
    };
    cmd_err_panic("chmod 755 ".to_string() + dir_name);
}

fn config_on_linux(tinc_home: &str) {
    let dev = get_wan_name();
    let config;

    if file_exists(&(tinc_home.to_string() + "/bridge.config")) {
        config = read_file(tinc_home.to_string() + "/bridge.config");
    } else {
        config = dev.unwrap();
    };

    cmd_err_panic(
        "sudo sed -i 's/\"wan_interface\":\".*\"/\"wan_interface\":\"".to_string() +
        &config.clone() + "\"/g\' " + tinc_home + "/register_config_json");

    cmd_err_panic("sudo sed -i 's/eth0/".to_string() + &config + "/g' " + tinc_home + "/tinc-up");

    cmd_err_panic("sudo sed -i 's/10.255.255.254/11.255.255.254/g' ".to_string() + tinc_home + "/tinc-up");

    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "/tinc-up");

    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "/tinc");

    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "/tincd");

    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "/start");

    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "/landmark");
}

pub fn create_pub_key(tinc_home: &str, pub_key_path: &str) {
    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "key/build-key-tinc.exp");
    cmd_err_panic(tinc_home.to_string() + "key/build-key-tinc.exp " + tinc_home
        + "key/rsa_key.priv " + tinc_home + pub_key_path);
}

#[test]
fn test_cp() {
    cp_tinc_to_local("/root/tinc");
}