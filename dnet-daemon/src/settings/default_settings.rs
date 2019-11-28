// common
pub const DEFAULT_LOG_LEVEL: &str = "Error";

// proxy
#[cfg(linux)]
pub const DEFAULT_LINUX_DEFAULT_HOME_PATH: &str = "/opt/dnet";
pub const DEFAULT_PROXY_LOCAL_SERVER_PORT: u16 = 443;
pub const DEFAULT_PROXY_TYPE: &str = "other";
pub const DEFAULT_CLIENT_AUTO_CONNECT: bool = true;
pub const HEARTBEAT_FREQUENCY_SEC: u32 = 20;
pub const DEFAULT_PROXY_PUBLIC: bool = false;

// tinc
pub const TINC_INTERFACE: &str = "dnet";
