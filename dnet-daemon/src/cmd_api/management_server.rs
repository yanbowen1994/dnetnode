use jsonrpc_core::{
    futures::{
        future,
        sync::{self, oneshot::Sender as OneshotSender},
        Future,
    },
    Error, MetaIoHandler, Metadata,
};
use jsonrpc_ipc_server;
use jsonrpc_macros::{build_rpc_trait, metadata, pubsub};
use jsonrpc_pubsub::{PubSubHandler, PubSubMetadata, Session, SubscriptionId};

use parking_lot::{Mutex, RwLock};
use std::{
    collections::HashMap,
    sync::Arc,
};
use ipc_server;
use dnet_types::states::{TunnelState, State};
use dnet_types::daemon_broadcast::DaemonBroadcast;
use dnet_types::response::Response;

use crate::cmd_api::types::EventListener;
use crate::mpsc::IntoSender;
use crate::settings::Settings;
use dnet_types::team::Team;
use dnet_types::tinc_host_status_change::HostStatusChange;
use dnet_types::user::User;

/// FIXME(linus): This is here just because the futures crate has deprecated it and jsonrpc_core
/// did not introduce their own yet (https://github.com/paritytech/jsonrpc/pull/196).
/// Remove this and use the one in jsonrpc_core when that is released.
pub type BoxFuture<T, E> = Box<dyn Future<Item = T, Error = E> + Send>;

build_rpc_trait! {
    pub trait ManagementInterfaceApi {
        type Metadata;
        #[rpc(meta, name = "tunnel_connect")]
        fn tunnel_connect(&self, Self::Metadata, String) -> BoxFuture<Response, Error>;

        #[rpc(meta, name = "tunnel_disconnect")]
        fn tunnel_disconnect(&self, Self::Metadata, String) -> BoxFuture<Response, Error>;

        #[rpc(meta, name = "shutdown")]
        fn shutdown(&self, Self::Metadata) -> BoxFuture<Response, Error>;

        #[rpc(meta, name = "status")]
        fn status(&self, Self::Metadata) -> BoxFuture<State, Error>;

        #[rpc(meta, name = "group_info")]
        fn group_info(&self, Self::Metadata, String) -> BoxFuture<Vec<Team>, Error>;

        #[rpc(meta, name = "group_users")]
        fn group_users(&self, Self::Metadata, String) -> BoxFuture<Response, Error>;

        #[rpc(meta, name = "group_join")]
        fn group_join(&self, Self::Metadata, String) -> BoxFuture<Response, Error>;

        #[rpc(meta, name = "group_out")]
        fn group_out(&self, Self::Metadata, String) -> BoxFuture<Response, Error>;

        #[rpc(meta, name = "group_list")]
        fn group_list(&self, Self::Metadata) -> BoxFuture<Vec<Team>, Error>;

        #[rpc(meta, name = "login")]
        fn login(&self, Self::Metadata, String) -> BoxFuture<Response, Error>;

        #[rpc(meta, name = "logout")]
        fn logout(&self, Self::Metadata) -> BoxFuture<Response, Error>;

        #[rpc(meta, name = "host_status_change")]
        fn host_status_change(&self, Self::Metadata, String) -> BoxFuture<(), Error>;
    }
}

/// Enum representing commands coming in on the management interface.
pub enum ManagementCommand {
    TeamConnect(OneshotSender<Response>, String),

    TeamDisconnect(OneshotSender<Response>, String),

    /// Request the current state.
    State(OneshotSender<State>),

    /// Get the current geographical location.
    GroupList(OneshotSender<Vec<Team>>),

    /// Get the current geographical location.
    GroupInfo(OneshotSender<Vec<Team>>, String),

    /// Get the current geographical location.
    GroupUsers(OneshotSender<Response>, String),

    /// Get the current geographical location.
    GroupJoin(OneshotSender<Response>, String),

    /// Get the current geographical location.
    GroupOut(OneshotSender<Response>, String),

    HostStatusChange(OneshotSender<()>, HostStatusChange),

    Login(OneshotSender<Response>, User),

    Logout(OneshotSender<Response>),

    Shutdown(OneshotSender<Response>),
}

pub struct ManagementInterfaceServer {
    server: ipc_server::IpcServer,
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, pubsub::Sink<DaemonBroadcast>>>>,
}

impl ManagementInterfaceServer {
    pub fn start<T>(path: &str, tunnel_tx: IntoSender<ManagementCommand, T>) -> Result<Self, ipc_server::Error>
        where
            T: From<ManagementCommand> + 'static + Send,
    {
        let rpc = ManagementInterface::new(tunnel_tx);
        let subscriptions = rpc.subscriptions.clone();

        let mut io = PubSubHandler::default();
        io.extend_with(rpc.to_delegate());
        let meta_io: MetaIoHandler<Meta> = io.into();
        let server = ipc_server::IpcServer::start_with_metadata(
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
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, pubsub::Sink<DaemonBroadcast>>>>,
    close_handle: Option<ipc_server::CloseHandle>,
}

impl EventListener for ManagementInterfaceEventBroadcaster {
    /// Sends a new state update to all `new_state` subscribers of the management interface.
    fn notify_new_state(&self, new_state: TunnelState) {
        log::info!("Broadcasting new state: {:?}", new_state);
        self.notify(DaemonBroadcast::TunnelState(new_state));
    }

    /// Sends settings to all `settings` subscribers of the management interface.
    fn notify_settings(&self, settings: Settings) {
        log::info!("Broadcasting new settings");
        self.notify(DaemonBroadcast::Settings(settings.into()));
    }
}

impl ManagementInterfaceEventBroadcaster {
    fn notify(&self, value: DaemonBroadcast) {
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
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, pubsub::Sink<DaemonBroadcast>>>>,
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
}

impl<T: From<ManagementCommand> + 'static + Send> ManagementInterfaceApi
for ManagementInterface<T>
{
    type Metadata = Meta;

    fn tunnel_connect(&self, _: Self::Metadata, team_id: String)
                      -> BoxFuture<Response, Error>
    {
        log::info!("management interface tunnel connect");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::TeamConnect(tx, team_id))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn tunnel_disconnect(&self, _: Self::Metadata, team_id: String)
                         -> BoxFuture<Response, Error>
    {
        log::info!("management interface tunnel disconnect");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::TeamDisconnect(tx, team_id))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn shutdown(&self, _: Self::Metadata) -> BoxFuture<Response, Error> {
        log::info!("management interface shutdown command.");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::Shutdown(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn status(&self, _: Self::Metadata) -> BoxFuture<State, Error> {
        log::info!("management interface get status.");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::State(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn group_list(&self, _: Self::Metadata) -> BoxFuture<Vec<Team>, Error> {
        log::info!("management interface group list");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GroupList(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn login(&self, _: Self::Metadata, user: String) -> BoxFuture<Response, Error> {
        log::info!("management interface login");
        let user = serde_json::from_str(&user).unwrap();
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::Login(tx, user))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn logout(&self, _: Self::Metadata) -> BoxFuture<Response, Error> {
        log::info!("management interface logout");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::Logout(tx))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn group_info(&self, _: Self::Metadata, team_id: String) -> BoxFuture<Vec<Team>, Error> {
        log::info!("management interface group info");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GroupInfo(tx, team_id))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn group_users(&self, _: Self::Metadata, team_id: String) -> BoxFuture<Response, Error> {
        log::info!("management interface group info");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GroupUsers(tx, team_id))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn group_join(&self, _: Self::Metadata, team_id: String) -> BoxFuture<Response, Error> {
        log::info!("management interface group join.");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GroupJoin(tx, team_id))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn group_out(&self, _: Self::Metadata, team_id: String) -> BoxFuture<Response, Error> {
        log::info!("management interface group join.");
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::GroupOut(tx, team_id))
            .and_then(|_| rx.map_err(|_| Error::internal_error()));
        Box::new(future)
    }

    fn host_status_change(&self, _: Self::Metadata, host_status_change: String)
                          -> BoxFuture<(), Error>
    {
        log::info!("management interface host status change.");

        let host_status_change = serde_json::from_str(&host_status_change).unwrap();
        let (tx, rx) = sync::oneshot::channel();
        let future = self
            .send_command_to_daemon(ManagementCommand::HostStatusChange(tx, host_status_change))
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