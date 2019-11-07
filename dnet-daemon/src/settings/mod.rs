extern crate config;

pub(crate) mod default_settings;
pub mod error;
mod parse_file;
mod run_time_settings;

pub use error::Error;
pub use run_time_settings::{Settings, get_settings, get_mut_settings};