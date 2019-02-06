//! tinc相关的操作

pub mod install_tinc;
pub mod check;
pub mod operator;
pub use self::check::check_tinc_complete;
pub use self::operator::Tinc;
pub use self::install_tinc::install_tinc;