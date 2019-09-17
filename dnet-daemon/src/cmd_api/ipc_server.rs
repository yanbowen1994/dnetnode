use jsonrpc_core::{
    futures::{
        future,
        sync::{self, oneshot::Sender as OneshotSender},
        Future,
    },
    Error, ErrorCode, MetaIoHandler, Metadata,
};
use jsonrpc_ipc_server;
use jsonrpc_macros::{build_rpc_trait, metadata, pubsub};
use jsonrpc_pubsub::{PubSubHandler, PubSubMetadata, Session, SubscriptionId};

use parking_lot::{Mutex, RwLock};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};
use talpid_ipc;
use crate::settings::Settings;
use crate::cmd_api::types::EventListener;
use crate::mpsc::IntoSender;
use dnet_types::states::TunnelState;
use dnet_types::daemon_broadcast::DaemonBroadcast;

/// FIXME(linus): This is here just because the futures crate has deprecated it and jsonrpc_core
/// did not introduce their own yet (https://github.com/paritytech/jsonrpc/pull/196).
/// Remove this and use the one in jsonrpc_core when that is released.
pub type BoxFuture<T, E> = Box<dyn Future<Item = T, Error = E> + Send>;

build_rpc_trait! {
    pub trait ManagementInterfaceApi {
        type Metadata;
        #[rpc(meta, name = "tunnel_connect")]
        fn tunnel_connect(&self, Self::Metadata) -> BoxFuture<(), Error>;

        #[rpc(meta, name = "tunnel_disconnect")]
        fn tunnel_disconnect(&self, Self::Metadata) -> BoxFuture<(), Error>;

        #[rpc(meta, name = "tunnel_status")]
        fn tunnel_status(&self, Self::Metadata) -> BoxFuture<(), Error>;

        #[rpc(meta, name = "rpc_status")]
        fn rpc_status(&self, Self::Metadata) -> BoxFuture<(), Error>;

        #[rpc(meta, name = "group_info")]
        fn group_info(&self, Self::Metadata, String) -> BoxFuture<(), Error>;
    }
}

/// Enum representing commands coming in on the management interface.
pub enum ManagementCommand {
    TunnelConnect(
        OneshotSender<()>,
    ),

    TunnelDisConnect(
        OneshotSender<()>,
    ),

    /// Change target state.
    TunnelStatus(
        OneshotSender<()>,
    ),

    /// Request the current state.
    RpcStatus(OneshotSender<()>),

    /// Get the current geographical location.
    GroupInfo(OneshotSender<()>, String),

    Shutdown,
}

pub struct ManagementInterfaceServer {
    server: talpid_ipc::IpcServer,
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, pubsub::Sink<DaemonBroadcast<Settings>>>>>,
}

impl ManagementInterfaceServer {
    pub fn start<T>(path: &str, tunnel_tx: IntoSender<ManagementCommand, T>) -> Result<Self, talpid_ipc::Error>
        where
            T: From<ManagementCommand> + 'static + Send,
    {
        let rpc = ManagementInterface::new(tunnel_tx);
        let subscriptions = rpc.subscriptions.clone();

        let mut io = PubSubHandler::default();
        io.extend_with(rpc.to_delegate());
        let meta_io: MetaIoHandler<Meta> = io.into();
        let server = talpid_ipc::IpcServer::start_with_metadata(
            meta_io,
            meta_extractor,
            &path,
        )?;
        Ok(ManagementInterfaceServer {
            server,
            subscriptions,
        })
    }

    pub fn socket_path(&self) -> &str {
        self.server.path()
    }

    pub fn event_broadcaster(&self) -> ManagementInterfaceEventBroadcaster {
        ManagementInterfaceEventBroadcaster {
            subscriptions: self.subscriptions.clone(),
            close_handle: Some(self.server.close_handle()),
        }
    }

    /// Consumes the server and waits for it to finish. Returns an error if the server exited
    /// due to an error.
    pub fn wait(self) {
        self.server.wait()
    }
}

/// A handle that allows broadcasting messages to all subscribers of the management interface.
#[derive(Clone)]
pub struct ManagementInterfaceEventBroadcaster {
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, pubsub::Sink<DaemonBroadcast<Settings>>>>>,
    close_handle: Option<talpid_ipc::CloseHandle>,
}

impl EventListener for ManagementInterfaceEventBroadcaster {
    /// Sends a new state update to all `new_state` subscribers of the management interface.
    fn notify_new_state(&self, new_state: TunnelState) {
        log::debug!("Broadcasting new state: {:?}", new_state);
        self.notify(DaemonBroadcast::TunnelState(new_state));
    }

    /// Sends settings to all `settings` subscribers of the management interface.
    fn notify_settings(&self, settings: Settings) {
        log::debug!("Broadcasting new settings");
        self.notify(DaemonBroadcast::Settings(settings));
    }
}

impl ManagementInterfaceEventBroadcaster {
    fn notify(&self, value: DaemonBroadcast<Settings>) {
        let subscriptions = self.subscriptions.read();
        for sink in subscriptions.values() {
            let _ = sink.notify(Ok(value.clone())).wait();
        }
    }
}

impl Drop for ManagementInterfaceEventBroadcaster {
    fn drop(&mut self) {
        if let Some(close_handle) = self.close_handle.take() {
            close_handle.close();
        }
    }
}

struct ManagementInterface<T: From<ManagementCommand> + 'static + Send> {
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, pubsub::Sink<DaemonBroadcast<Settings>>>>>,
    tx: Mutex<IntoSender<ManagementCommand, T>>,
}

impl<T: From<ManagementCommand> + 'static + Send> ManagementInterface<T> {
    pub fn new(tx: IntoSender<ManagementCommand, T>) -> Self {
        ManagementInterface {
            subscriptions: Default::default(),
            tx: Mutex::new(tx),
        }
    }

    /// Sends a command to the daemon and maps the error to an RPC error.
    fn send_command_to_daemon(
        &self,
        command: ManagementCommand,
    ) -> impl Future<Item = (), Error = Error> {
        future::result(self.tx.lock().send(command)).map_err(|_| Error::internal_error())
    }

    /// Converts the given error to an error that can be given to the caller of the API.
    /// Will let any actual RPC error through as is, any other error is changed to an internal
    /// error.
    fn map_rpc_error(error: &jsonrpc_core::Error) -> Error {
//        match error.kind() {
//            Error::JsonRpcError(ref rpc_error) => {
//                // We have to manually copy the error since we have different
//                // versions of the jsonrpc_core library at the moment.
//                Error {
//                    code: ErrorCode::from(rpc_error.code.code()),
//                    message: rpc_error.message.clone(),
//                    data: rpc_error.data.clone(),
//                }
//            }
//            _ => Error::internal_error(),
//        }
        Error::internal_error()
    }
}

impl<T: From<ManagementCommand> + 'static + Send> ManagementInterfaceApi
for ManagementInterface<T>
{
    type Metadata = Meta;

    fn tunnel_connect(&self, _: Self::Metadata) -> BoxFuture<(), Error> {
        log::debug!("create_account");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::TunnelConnect(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn tunnel_disconnect(&self,
                         _: Self::Metadata) -> BoxFuture<(), Error> {
        log::debug!("create_account");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::TunnelDisConnect(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn tunnel_status(&self,
                     _: Self::Metadata) -> BoxFuture<(), Error> {
        log::debug!("create_account");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::TunnelStatus(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn rpc_status(&self,
                  _: Self::Metadata) -> BoxFuture<(), Error> {
        log::debug!("create_account");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::RpcStatus(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn group_info(&self, _: Self::Metadata, id: String) -> BoxFuture<(), Error> {
        log::debug!("create_account");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GroupInfo(tx, id))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }
}


/// The metadata type. There is one instance associated with each connection. In this pubsub
/// scenario they are created by `meta_extractor` by the server on each new incoming
/// connection.
#[derive(Clone, Debug, Default)]
pub struct Meta {
    session: Option<Arc<Session>>,
}

/// Make the `Meta` type possible to use as jsonrpc metadata type.
impl Metadata for Meta {}

/// Make the `Meta` type possible to use as a pubsub metadata type.
impl PubSubMetadata for Meta {
    fn session(&self) -> Option<Arc<Session>> {
        self.session.clone()
    }
}

/// Metadata extractor function for `Meta`.
fn meta_extractor(context: &jsonrpc_ipc_server::RequestContext<'_>) -> Meta {
    Meta {
        session: Some(Arc::new(Session::new(context.sender.clone()))),
    }
}
