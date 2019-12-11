pub mod setting;

use crate::Command;
use std::collections::HashMap;

mod connect;
pub use self::connect::Connect;

mod disconnect;
pub use self::disconnect::Disconnect;

mod group;
pub use self::group::Group;

mod login;
pub use self::login::Login;

mod logout;
pub use self::logout::Logout;

mod shutdown;
pub use self::shutdown::Shutdown;

mod status;
pub use self::status::Status;

/// Returns a map of all available subcommands with their name as key.
pub fn get_commands() -> HashMap<&'static str, Box<dyn Command>> {
    let mut map = HashMap::new();
    let commands: Vec<Box<dyn Command>> = vec![
        Box::new(Connect),
        Box::new(Disconnect),
        Box::new(Group),
        Box::new(Login),
        Box::new(Logout),
        Box::new(Shutdown),
        Box::new(Status),
    ];
    for cmd in commands {
        if map.insert(cmd.name(), cmd).is_some() {
            panic!("Multiple commands with the same name");
        }
    }
    map
}
