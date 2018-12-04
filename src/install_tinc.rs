use s;
//use super::file_tool;

//use std::string;

fn install_tinc() {
    install_on_linux();
}

fn install_on_linux() -> i32 {
    cp_tinc_to_local();
    add_dependent("liblzo2-2");
    add_permission_dir(TINC_HONE);
    config_on_linux();
    create_pub_key();
    if ! is_tinc_exist() {
        start_tinc();
        if is_tinc_exist() {
            stop_tinc();
            println!("Success install tinc");
        } else {
            println!("Failed install tinc");
            return -1;
        }
    } else {
        println!("tinc is running");
    }
    install_landmark();
    return 0;
}

fn cp_tinc_to_local() -> bool {
    let (code, output) = sys_tool::cmd("rm -rf /root/tinc".to_string());
    if code != 0 {
        println!("{}", output);
        return false;
    }
    let (code, output) = sys_tool::cmd("cp -rf ./tinc /root/tinc".to_string());
    if code != 0 {
        println!("{}", output);
        return false;
    }
    return true;
}

fn add_dependent(app_name:&str) -> bool {
    if "Ubuntu" == &(sys_tool::current_platform().1) {
        let (code, output) = sys_tool::cmd(
            "sudo dpkg --get-selections | grep " + app_name);
        if code != 0 {
            println!("{}", output);
            return false;
        }
        println!("**** install Dependency package " + app_name);
        let (code, output) = sys_tool::cmd(
            "sudo apt-get install -y " + app_name);
        if code != 0 {
            println!("{}", output);
            return false;
        }
    }
    return true;
}

fn add_permission_dir(dir_name:&str) ->bool {
    if !file_tool::dir_exists() {
        return false;
    };
    let (code, output) = sys_tool::cmd(
        "chmod 755 " + dir_name);
    if code != 0 {
        println!("{}", output);
        return false;
    }
    return true;
}

fn config_on_linux() -> bool {

}

















#[test]
fn test_cp() {
    cp_tinc_to_local();
}