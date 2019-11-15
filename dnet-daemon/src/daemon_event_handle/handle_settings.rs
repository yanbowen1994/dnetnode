use std::sync::{mpsc};

use url;
use futures::sync::oneshot;

use dnet_types::response::Response;
use crate::settings::get_settings;
use crate::daemon::Daemon;

pub fn check_conductor_url(
    ipc_tx: oneshot::Sender<Response>
) -> Option<oneshot::Sender<Response>> {
    let settings = get_settings();
    let conductor = settings.common.conductor_url.clone();
    if let Ok(_) = url::Url::parse(&conductor) {
        Some(ipc_tx)
    }
    else {
        let response = Response::internal_error().set_msg("Conductor url failed.".to_owned());
        let _ = Daemon::oneshot_send(ipc_tx, response, "");
        None
    }
}