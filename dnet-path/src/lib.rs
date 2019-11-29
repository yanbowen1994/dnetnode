use std::path::PathBuf;

extern crate dirs;
extern crate log;

#[cfg(target_os = "linux")]
pub fn home_dir(linux_path: Option<&str>) -> Option<PathBuf> {
    if let Some(linux_path) = linux_path {
        return Some(PathBuf::from(linux_path));
    }
    return Some(PathBuf::from("/opt/dnet"));
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn home_dir() -> Option<PathBuf> {
    return dirs::home_dir();
}

pub fn ipc_path() -> String {
    #[cfg(not(target_os = "windows"))]
        {
            "/tmp/.dnet.socket".to_owned()
        }
    #[cfg(target_os = "windows")]
        {
            "//./pipe/dnet".to_owned()
        }
}