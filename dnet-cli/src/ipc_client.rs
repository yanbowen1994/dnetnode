extern crate talpid_ipc;

use ipc_client::new_standalone_ipc_client;
use jsonrpc_client_core::ClientHandle;
use serde::{Serialize, Deserialize};
use futures::future::Future;

pub fn new_ipc_client() {
    // TODO dnet path
    match new_standalone_ipc_client("/opt/dnet/dnet.socket") {
        Err(e) => Err(Error::DaemonNotRunning(e)),
        Ok(client) => Ok(client),
    }
}

pub fn call<A, O>(handle: ClientHandle, method: &'static str, args: &A)
    where
        A: Serialize + Send + 'static,
        O: for<'de> Deserialize<'de> + Send + 'static,
{
    handle.call_method(method, args).wait();
}