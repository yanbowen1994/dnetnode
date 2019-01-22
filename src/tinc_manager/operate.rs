use sys_tool::{cmd_err_panic, cmd};

pub fn start_tinc(tinc_home: &str) {
    cmd_err_panic(tinc_home.to_string() + "/start");
}

pub fn stop_tinc() {
    cmd_err_panic("killall tincd".to_string());
}

pub fn is_tinc_exist(tinc_home: &str) -> bool {
    let (code, output) = cmd(
        "sudo ps aux | grep ".to_string() + tinc_home + "tincd | grep -v 'grep'");

    if output.len() > 0 {
        return true;
    };
    return false;
}

pub fn restart_tinc(tinc_home: &str) {
    for i in 0..3 {
        if is_tinc_exist(tinc_home) {
            stop_tinc();
        }
        start_tinc(tinc_home);
        if !is_tinc_exist(tinc_home) {
            if i == 2 {
                panic!("Error: Fail to restart tinc.");
            }
        } else {
            break;
        }
    }
}