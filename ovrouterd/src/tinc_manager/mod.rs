//! tinc相关的操作

pub mod check;
mod control;
pub mod operator;

pub use self::control::tinc_connections;
pub use self::operator::TincOperator;
