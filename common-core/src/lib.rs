#[macro_use]
extern crate serde;

mod logging;
pub use logging::init_logger;

mod settings;
pub use settings::{Settings, get_settings, Error as SettingsError};