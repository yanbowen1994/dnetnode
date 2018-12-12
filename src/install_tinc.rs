use file_tool::*;
use sys_tool::*;
use net_tool::{get_wan_name};
use global_constant::*;

pub fn install_tinc() {
    install_on_linux();
}

fn install_on_linux() -> i32 {
    cp_tinc_to_local();
    add_dependent("liblzo2-2");
    add_permission_dir(TINC_HONE);
    config_on_linux();
    create_pub_key();
    if !is_tinc_exist() {
        start_tinc();
        if is_tinc_exist() {
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

fn cp_tinc_to_local() {
    cmd_err_panic("rm -rf /root/tinc".to_string());

    cmd_err_panic("cp -rf ./tinc /root/tinc".to_string());
}

fn add_dependent(app_name:&str) {
    if "Ubuntu" == &(current_platform().1) {
        cmd_err_panic("sudo dpkg --get-selections | grep ".to_string() + app_name);

        println!("{}", "**** install Dependency package ".to_string() + app_name);
        cmd_err_panic("sudo apt-get install -y ".to_string() + app_name);
    }
}

fn add_permission_dir(dir_name:&str) {
    if !dir_exists(dir_name.to_string()) {
        panic!("Dir {} not exists", dir_name);
    };
    cmd_err_panic("chmod 755 ".to_string() + dir_name);
}

fn config_on_linux() {
    let dev = get_wan_name();
    let config;

    if file_exists(TINC_HONE.to_string() + "/bridge.config") {
        config = read_file(TINC_HONE.to_string() + "/bridge.config");
    } else {
        config = dev.unwrap();
    };

    cmd_err_panic(
        "sudo sed -i 's/\"wan_interface\":\".*\"/\"wan_interface\":\"".to_string() +
        &config.clone() + "\"/g\' " + TINC_HONE + "/register_config_json");

    cmd_err_panic("sudo sed -i 's/eth0/".to_string() + &config + "/g' " + TINC_HONE + "/tinc-up");

    cmd_err_panic("sudo sed -i 's/10.255.255.254/11.255.255.254/g' ".to_string() + TINC_HONE + "/tinc-up");

    cmd_err_panic("chmod 755 ".to_string() + TINC_HONE + "/tinc-up");

    cmd_err_panic("chmod 755 ".to_string() + TINC_HONE + "/tinc");

    cmd_err_panic("chmod 755 ".to_string() + TINC_HONE + "/tincd");

    cmd_err_panic("chmod 755 ".to_string() + TINC_HONE + "/start");

    cmd_err_panic("chmod 755 ".to_string() + TINC_HONE + "/landmark");
}

fn create_pub_key() {
    cmd_err_panic("chmod 755 ".to_string() + TINC_HONE + "key/build-key-tinc.exp");
    cmd_err_panic(TINC_HONE.to_string() + "key/build-key-tinc.exp " + TINC_HONE
        + "key/rsa_key.priv " + TINC_HONE + TINC_RSAKYE_PATH);
}

fn start_tinc() {
    cmd_err_panic(TINC_HONE.to_string() + "/start");
}

fn stop_tinc() {
    cmd_err_panic("killall tincd".to_string());
}

fn is_tinc_exist() -> bool {
    let (code, output) = cmd(
        "sudo ps aux | grep ".to_string() + TINC_HONE + "tincd | grep -v 'grep'");

    if output.len() > 0 {
      return true;
    };
    return false;
}

fn restart_tinc() {
    for i in 0..3 {
        if is_tinc_exist() {
            stop_tinc();
        }
        start_tinc();
        if !is_tinc_exist() {
            if i == 2 {
                panic!("Error: Fail to restart tinc.");
            }
        } else {
            break;
        }
    }
}



#[test]
fn test_cp() {
    cp_tinc_to_local();
}