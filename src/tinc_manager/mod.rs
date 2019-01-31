//! tinc相关的操作

pub mod install_tinc;
pub mod check;
pub mod operater;
pub use self::check::check_tinc_complete;
pub use self::operater::Tinc;
pub use self::install_tinc::install_tinc;