use std::process::Command;
use std::str;

extern crate os_type;
use self::os_type::*;

pub fn cmd_err_panic(command:String) -> String {
    let (code, output) = cmd(command);
    if code != 0 {
        panic!(output);
    };
    return output;
}

#[cfg(target_os = "linux")]
pub fn cmd(command:String) -> (i32, String) {
    let res;
    let output: String;
    let code: i32;
    let command_clone= command.clone();

    res = Command::new("/bin/bash")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|err| err.to_string());

    match res {
        Ok(pres) => {
            code = pres.status.code().expect(&("can't exec ".to_string() + &command_clone));
            let err:String = str::from_utf8(&pres.stderr).ok().unwrap().to_owned();
            if err.len() > 0 {
                output = err.clone();
            } else {
                output = str::from_utf8(&pres.stdout).ok().unwrap().to_owned();
            };
        },

        Err(err) => {
            output = err;
            code = -1;
        },
    };

    return (code, output.replace("\n", ""));
}

#[cfg(target_os = "linux")]
pub fn current_platform() -> (String, String) {
    let platform = os_type::current_platform();
    let version:String = platform.version;
    let plt = match platform.os_type {
        OSType::Unknown => "Unknown",
        OSType::Redhat => "Redhat",
        OSType::OSX => "OSX",
        OSType::Ubuntu => "Ubuntu",
        OSType::Debian => "Debian",
        OSType::Arch => "Arch",
        OSType::Manjaro => "Manjaro",
        OSType::CentOS => "CentOS",
        OSType::OpenSUSE => "OpenSUSE",
    };

    return (plt.to_string(), version)
}