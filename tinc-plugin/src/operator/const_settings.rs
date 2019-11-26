#[cfg(unix)]
mod unix {
    pub const TINC_BIN_FILENAME: &str = "tincd";

    pub const TINC_UP_FILENAME: &str = "tinc-up";
}
#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
mod windows {
    pub const TINC_BIN_FILENAME: &str = "tincd.exe";

    pub const TINC_UP_FILENAME: &str = "tinc-up.bat";
}
#[cfg(windows)]
pub use windows::*;

pub const PRIV_KEY_FILENAME: &str = "rsa_key.priv";

pub const PUB_KEY_FILENAME: &str = "rsa_key.pub";

pub const PID_FILENAME: &str = "tinc.pid";

pub const TINC_MEMORY_LIMIT: f32 = (85 as f32);
// if per 3 seconds check. out of memory over 15 second. Error::OutOfMemory
pub const TINC_ALLOWED_OUT_MEMORY_TIMES: u32 = 5;

pub const DEFAULT_TINC_PORT: u16 = (50069 as u16);