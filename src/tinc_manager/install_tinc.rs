use file_tool::*;
use sys_tool::*;
use net_tool::{get_wan_name};
use super::operator::*;
use settings::Settings;

pub fn install_tinc(settings: &Settings, tinc: &Tinc) {
    install_on_linux(settings, tinc);
}

fn install_on_linux(settings: &Settings, tinc: &Tinc) -> i32 {
    let tinc_home  = &settings.tinc.home_path[..];
    cp_tinc_to_local(tinc_home);
    add_dependent("liblzo2-2");
    add_dependent("libcurl4-openssl-dev");
    add_permission_dir(tinc_home);
    config_on_linux(tinc_home);
    tinc.create_pub_key();
    if !tinc.is_tinc_exist() {
        tinc.start_tinc();
        if tinc.is_tinc_exist() {
            tinc.stop_tinc();
            info!("{}", "Success install tinc");
        } else {
            info!("{}", "Failed install tinc");
            return -1;
        }
    } else {
        info!("tinc is running");
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

        info!("{}", "**** install Dependency package ".to_string() + app_name);
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

    cmd_err_panic(
        "sudo sed -i 's/\"wan_interface\":\".*\"/\"wan_interface\":\"".to_string() +
        &config.clone() + "\"/g\' " + tinc_home + "/register_config_json");

    cmd_err_panic("sudo sed -i 's/eth0/".to_string() + &config + "/g' " + tinc_home + "/tinc-up");

    cmd_err_panic("sudo sed -i 's/10.255.255.254/11.255.255.254/g' ".to_string() + tinc_home + "/tinc-up");

    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "/tinc-up");

    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "/tinc");

    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "/tincd");

    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "/start");

//    cmd_err_panic("chmod 755 ".to_string() + tinc_home + "/landmark");
}

#[test]
fn test_cp() {
    let settings = Settings::load_config().unwrap();
    let tinc = Tinc::new(
        settings.tinc.home_path.clone(),
        settings.tinc.pub_key_path.clone());
    install_tinc(&settings, &tinc);
}