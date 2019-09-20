mod auth;
mod client_info;
mod error;
mod info;
mod proxy_info;
mod tinc;

pub use self::error::Error;
pub use self::auth::AuthInfo;
pub use self::info::Info;
pub use self::client_info::ClientInfo;
pub use self::proxy_info::ProxyInfo;
pub use self::tinc::TincInfo;
pub use self::info::{get_info, get_mut_info};