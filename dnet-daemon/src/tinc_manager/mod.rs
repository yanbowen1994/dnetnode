//! tinc相关的操作

mod control;
pub mod operator;
mod tinc_monitor;

pub use self::control::tinc_connections;
pub use self::operator::TincOperator;
pub use self::tinc_monitor::TincMonitor;
