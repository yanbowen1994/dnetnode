use std::{io, path::Path, thread};

use futures::sync::oneshot;
use jsonrpc_client_core::{Client, ClientHandle, Future};
pub use jsonrpc_client_core::Error;
use jsonrpc_client_ipc::IpcTransport;
use serde::{Deserialize, Serialize};
pub use jsonrpc_client_pubsub::Error as PubSubError;

use dnet_types::daemon_broadcast::DaemonBroadcast;
use dnet_types::states::State;
use dnet_types::response::Response;
use dnet_types::team::Team;

pub type Result<T> = std::result::Result<T, Error>;

static NO_ARGS: [u8; 0] = [];

pub fn new_standalone_ipc_client(path: &impl AsRef<Path>) -> io::Result<DaemonRpcClient> {
    let path = path.as_ref().to_string_lossy().to_string();

    new_standalone_transport(path, |path| {
        IpcTransport::new(&path, &tokio::reactor::Handle::default())
    })
}

pub fn new_standalone_transport<
    F: Send + 'static + FnOnce(String) -> io::Result<T>,
    T: jsonrpc_client_core::DuplexTransport + 'static,
>(
    rpc_path: String,
    transport_func: F,
) -> io::Result<DaemonRpcClient> {
    let (tx, rx) = oneshot::channel();
    thread::spawn(move || match spawn_transport(rpc_path, transport_func) {
        Err(e) => tx
            .send(Err(e))
            .expect("Failed to send error back to caller"),
        Ok((client, server_handle, client_handle)) => {
            let mut rt = tokio::runtime::current_thread::Runtime::new()
                .expect("Failed to start a standalone tokio runtime for mullvad ipc");
            let handle = rt.handle();
            tx.send(Ok((client_handle, server_handle, handle)))
                .expect("Failed to send client handle");

            if let Err(e) = rt.block_on(client) {
                log::error!("JSON-RPC client failed: {}", e.description());
            }
        }
    });

    rx.wait()
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "No transport handles returned"))?
        .map(|(rpc_client, server_handle, executor)| {
            let subscriber =
                jsonrpc_client_pubsub::Subscriber::new(executor, rpc_client.clone(), server_handle);
            DaemonRpcClient {
                rpc_client,
                subscriber,
            }
        })
}

fn spawn_transport<
    F: Send + FnOnce(String) -> io::Result<T>,
    T: jsonrpc_client_core::DuplexTransport + 'static,
>(
    address: String,
    transport_func: F,
) -> io::Result<(
    Client<T, jsonrpc_client_core::server::Server>,
    jsonrpc_client_core::server::ServerHandle,
    ClientHandle,
)> {
    let (server, server_handle) = jsonrpc_client_core::server::Server::new();
    let transport = transport_func(address)?;
    let (client, client_handle) = jsonrpc_client_core::Client::with_server(transport, server);
    Ok((client, server_handle, client_handle))
}

pub struct DaemonRpcClient {
    rpc_client: jsonrpc_client_core::ClientHandle,
    subscriber: jsonrpc_client_pubsub::Subscriber<tokio::runtime::current_thread::Handle>,
}


impl DaemonRpcClient {
    pub fn status(&mut self) -> Result<State> {
        self.call("status", &NO_ARGS)
    }

    pub fn tunnel_connect(&mut self, team_id: String) -> Result<Response> {
        self.call("tunnel_connect", &team_id)
    }

    pub fn tunnel_disconnect(&mut self, team_id: String) -> Result<()> {
        self.call("tunnel_disconnect", &team_id)
    }

    pub fn shutdown(&mut self) -> Result<Response> {
        self.call("shutdown", &NO_ARGS)
    }

    pub fn group_list(&mut self, ) -> Result<Vec<Team>> {
        self.call("group_list", &NO_ARGS)
    }

    pub fn group_info(&mut self, team_id: String) -> Result<Vec<Team>> {
        self.call("group_info", &team_id)
    }

    pub fn group_users(&mut self, team_id: String) -> Result<Response> {
        self.call("group_users", &team_id)
    }

    pub fn group_join(&mut self, team_id: String) -> Result<Response> {
        self.call("group_join", &team_id)
    }

    pub fn group_out(&mut self, team_id: String) -> Result<Response> {
        self.call("group_out", &team_id)
    }

    pub fn login(&mut self, user: String) -> Result<Response> {
        self.call("login", &user)
    }

    pub fn logout(&mut self) -> Result<Response> {
        self.call("logout", &NO_ARGS)
    }

    pub fn host_status_change(&mut self, host_status_change: String) -> Result<()> {
        self.call("host_status_change", &host_status_change)
    }

    pub fn call<A, O>(&mut self, method: &'static str, args: &A) -> Result<O>
        where
            A: Serialize + Send + 'static,
            O: for<'de> Deserialize<'de> + Send + 'static,
    {
        self.rpc_client.call_method(method, args).wait()
    }

    pub fn daemon_event_subscribe(
        &mut self,
    ) -> impl Future<
        Item = jsonrpc_client_pubsub::Subscription<DaemonBroadcast>,
        Error = jsonrpc_client_pubsub::Error,
    > {
        self.subscriber.subscribe(
            "daemon_event_subscribe".to_string(),
            "daemon_event_unsubscribe".to_string(),
            "daemon_event".to_string(),
            0,
            &NO_ARGS,
        )
    }
}
