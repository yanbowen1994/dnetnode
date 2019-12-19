#[cfg(unix)]
mod unix {
    pub const TINC_BIN_FILENAME: &str = "tincd";

    pub const TINC_UP_FILENAME: &str = "tinc-up";

    pub const TINC_DOWN_FILENAME: &str = "tinc-down";

    pub const HOST_UP_FILENAME: &str = "host-up";

    pub const HOST_DOWN_FILENAME: &str = "host-down";
}
#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
mod windows {
    pub const TINC_BIN_FILENAME: &str = "tincd.exe";

    pub const TINC_UP_FILENAME: &str = "tinc-up.bat";

    pub const TINC_DOWN_FILENAME: &str = "tinc-down.bat";

    pub const HOST_UP_FILENAME: &str = "host-up.bat";

    pub const HOST_DOWN_FILENAME: &str = "host-down.bat";
}
#[cfg(windows)]
pub use windows::*;

pub const PRIV_KEY_FILENAME: &str = "rsa_key.priv";

pub const PUB_KEY_FILENAME: &str = "rsa_key.pub";

pub const PID_FILENAME: &str = "tinc.pid";

pub const DEFAULT_TINC_PORT: u16 = (50069 as u16);