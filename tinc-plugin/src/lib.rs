//#[cfg(feature = "serde")]
//#[cfg_attr(feature = "serde", macro_use)]
extern crate serde;

#[macro_use]
extern crate serde_derive;

extern crate derive_try_from_primitive;

mod operator;
pub use operator::{TincOperator, Error as TincOperatorError};
mod info;
pub use info::{TincInfo, TincRunMode, ConnectTo};
pub mod tinc_tcp_stream;
pub mod control;
pub mod listener;
pub use self::listener::EventType;
pub use self::listener::spawn;
