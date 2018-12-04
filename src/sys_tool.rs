use std::string;
//For datetime
extern crate chrono;
use self::chrono::prelude::Local;

//For cmd
use std::process::Command;
use std::str;

//For current_platform
extern crate os_type;
use self::os_type::*;
extern crate regex;
use self::regex::Regex;
//extern crate encoding;
//use self::encoding::all::GB18030;
//use std::process::Command;

// This function only gets compiled if the target OS is linux
#[cfg(target_os = "linux")]
pub fn is_linux() ->bool {
    return true;
}

// And this function only gets compiled if the target OS is *not* linux
#[cfg(not(target_os = "linux"))]
pub fn is_linux() ->bool {
    return false;
}

#[cfg(target_os = "windows")]
pub fn is_win() ->bool {
    return true;
}

#[cfg(not(target_os = "windows"))]
pub fn is_win() ->bool {
    return false;
}

pub fn get_env_var(key:&str) -> String {
    let mut value:String = String::from("");
    value = match ::std::env::var(key) {
        Ok(value) => value,
        Err(_) => return value,
    };
    return value;
}

// windows下正常。 linux下非系统应用获取不到返回值，且无法获取外部程序执行错误。
// v2 Linux 应该可以了，windows下返回中文无法处理需要GBK库 暂时未处理
///如果成功返回code = 0, output=执行的stdout输出, 否则code = 错误码，output = 错误信息
pub fn cmd(command:String) -> (i32, String) {
    let res;
    let output: String;
    let code: i32;
    let command_clone= command.clone();

    if is_win() {
        let iter: Vec<_> = command.split_whitespace().collect();
        let list_command = iter.into_iter();

        res = Command::new("cmd")
            .arg("/C")
            .args(list_command)
            .output()
            .map_err(|err| err.to_string());
    } else {
        res = Command::new("/bin/bash")
            .arg("-c")
            .arg(command)
            .output()
            .map_err(|err| err.to_string());
    };
    match res {
        Ok(pres) => {
            code = pres.status.code().expect(&("can't exec ".to_string() + &command_clone));
            if is_win() {
                output = str::from_utf8(&pres.stdout).ok().unwrap().to_owned();
            } else {
                let err:String = str::from_utf8(&pres.stderr).ok().unwrap().to_owned();
                if err.len() > 0 {
                    output = err.clone();
                } else {
                    output = str::from_utf8(&pres.stdout).ok().unwrap().to_owned();
                };
            };
        },

        Err(err) => {
            output = err;
            code = -1;
        },
    };

    return (code, output.replace("\n", ""));
}

pub fn datetime() -> String {
    let dt = Local::now();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

// 用于获取Windows版本，又有cmd 无法获取中文，仅在英文windows下有效
struct WindowsVer {
    pub version: Option<String>
}

fn retrieve() -> Option<WindowsVer> {
    let output = match Command::new("ver").output() {
        Ok(o) => o,
        Err(_) => return None
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);
    Some(parse(stdout.to_string()))
}

fn parse(output: String) -> WindowsVer {
    let version_regex = Regex::new(r"^Microsoft Windows \[.*\s(\d+\.\d+\.\d+)\]$").unwrap();

    let version = match version_regex.captures_iter(&output).next() {
        Some(m) => {
            match m.get(1) {
                Some(version) => Some(version.as_str().to_owned()),
                None => None
            }
        },
        None => None
    };
    WindowsVer { version: version }
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

#[cfg(target_os = "windows")]
pub fn current_platform() -> (String, String) {
    let plt = "Windows";
    version = match retrieve() {
        Some(v) => v.version.unwrap().to_string(),
        None => "0.0.0".to_string(),
    };
    return (plt.to_string(), version)
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cmd() {
        if is_win() {
            let (a, b) = cmd(String::from("echo hello"));
            assert_eq!(a, 0);
            assert_eq!(&b, "hello\r\n");
        }
            else {
                let (a, b) = cmd(String::from("echo hello"));
                assert_eq!(a, 0);
                assert_eq!(&b, "hello\n");
            }
    }
    #[test]
    fn test_current_platform() {
        if is_win() {
            let (a, b) = current_platform();
            assert_eq!(a, "Windows".to_string());
            //中文版windows 无法获取ver信息：（Microsoft Windows [版本 6.1.7601]）0.0.0为不可识别
            assert_eq!(b, "0.0.0".to_string());
        } else {
            //test in Ubuntu 16.04
            let (a, b) = current_platform();
            assert_eq!(a, "Ubuntu".to_string());
            assert_eq!(b, "16.04".to_string());
        }

    }
}