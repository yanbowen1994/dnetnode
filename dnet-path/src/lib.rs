use std::path::PathBuf;

extern crate dirs;
#[macro_use]
extern crate log;

pub fn home_dir(linux_path: Option<&str>) -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        if let Some(linux_path) = linux_path {
            return Some(PathBuf::from(linux_path));
        }
        return Some(PathBuf::from("/opt/dnet"));
    }
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    return dirs::home_dir();

    warn!("Unkown operation_system. home_dir use /opt/dnet.");
    return Some(PathBuf::from("/opt/dnet"));
}