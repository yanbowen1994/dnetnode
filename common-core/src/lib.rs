#[macro_use]
extern crate err_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

mod bin_init;
pub use bin_init::{init, Error as InitError};

pub mod daemon;

mod logging;
pub use logging::init_logger;

pub mod traits;

mod settings;
pub use settings::{Settings, get_settings, Error as SettingsError};

mod shutdown;
pub use shutdown::set_shutdown_signal_handler;

pub mod net_tool;
//pub mod sys_tool;