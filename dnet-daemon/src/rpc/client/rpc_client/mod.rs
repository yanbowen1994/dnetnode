mod binding_device;
mod connect_team_broadcast;
mod device_select_proxy;
mod error;
mod get_online_proxy;
mod get_users_by_team;
mod heartbeat;
mod join_team;
mod login;
mod key_report;
mod out_team;
mod rpc_client;
mod search_team_by_mac;
mod search_team_handle;
mod search_user_team;
mod select_proxy;
pub(self) mod types;

pub use error::Error;
use error::Result;
pub use rpc_client::RpcClient;
pub(self) use rpc_client::{post};
pub use select_proxy::select_proxy;