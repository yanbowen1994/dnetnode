use jsonrpc_core::{
    futures::{
        future,
        sync::{self, oneshot::Sender as OneshotSender},
        Future,
    },
    Error, ErrorCode, MetaIoHandler, Metadata,
};
pub use serde_json::Value;
use dnet_types::states::TunnelState;
use crate::settings::Settings;

/// Trait representing something that can broadcast daemon events.
pub trait EventListener {
    /// Notify that the tunnel state changed.
    fn notify_new_state(&self, new_state: TunnelState);

    /// Notify that the settings changed.
    fn notify_settings(&self, settings: Settings);
}

//#[derive(Clone, Debug, Serialize, Deserialize)]
//pub struct CommandResponse {
//    pub code:   u32,
//    pub msg:    String,
//    #[serde(skip_serializing_if = "Option::is_none")]
//    pub data: Option<Value>
//}
//
//impl CommandResponse {
//    pub fn success() -> Self {
//        Self {
//            code: 200,
//            msg:  "".to_owned(),
//            data: None,
//        }
//    }
//
//    pub fn exec_timeout() -> Self {
//        Self {
//            code: 500,
//            msg:  "Internal Server Error".to_string(),
//            data: None,
//        }
//    }
//}