extern crate derive_try_from_primitive;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod operator;
pub use operator::{TincSettings, TincTools, TincOperator,
                   Error as TincOperatorError, PUB_KEY_FILENAME, PID_FILENAME, DEFAULT_TINC_PORT};
mod info;
pub mod tinc_tcp_stream;
pub mod control;
pub mod listener;
pub mod team;

pub use info::{TincInfo, TincRunMode, ConnectTo};
pub use listener::start;
pub use team::TincTeam;