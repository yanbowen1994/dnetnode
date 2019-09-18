pub mod setting;

use crate::Command;
use std::collections::HashMap;

mod connect;
pub use self::connect::Connect;

mod disconnect;
pub use self::disconnect::DisConnect;

mod status;
pub use self::status::Status;

mod tunnel;
pub use self::tunnel::Tunnel;

/// Returns a map of all available subcommands with their name as key.
pub fn get_commands() -> HashMap<&'static str, Box<dyn Command>> {
    let commands: Vec<Box<dyn Command>> = vec![
        Box::new(Status),
        Box::new(Connect),
        Box::new(DisConnect),
    ];
    let mut map = HashMap::new();
    for cmd in commands {
        if map.insert(cmd.name(), cmd).is_some() {
            panic!("Multiple commands with the same name");
        }
    }
    map
}