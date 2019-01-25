//pub mod proxy_info;

extern crate actix;
extern crate actix_web;
extern crate bytes;
extern crate futures;

pub mod server;
pub mod client;

pub use self::client::Client;
pub use self::server::web_server;