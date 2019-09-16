use std::{
    sync::{mpsc::{
        self, Receiver, Sender,
    }, Mutex},
    time::Duration,
};

use futures::{sync::oneshot, Future};
use jsonrpc_client_core::Transport;
use jsonrpc_core::{Error, IoHandler};

use jsonrpc_macros::build_rpc_trait;

use talpid_ipc::IpcServer;
use super::types::IpcCommand;
use super::types::CommandResponse;
use crate::daemon::DaemonEvent;

build_rpc_trait! {
    pub trait CommandApiTrait {
        #[rpc(name = "tunnel_connect")]
        fn tunnel_connect(&self) -> Result<CommandResponse, Error>;

        #[rpc(name = "tunnel_disconnect")]
        fn tunnel_disconnect(&self) -> Result<CommandResponse, Error>;

        #[rpc(name = "tunnel_status")]
        fn tunnel_status(&self) -> Result<CommandResponse, Error>;

        #[rpc(name = "rpc_status")]
        fn rpc_status(&self) -> Result<CommandResponse, Error>;

        #[rpc(name = "group_info")]
        fn group_info(&self, String) -> Result<CommandResponse, Error>;
    }
}

pub struct CommandInterface {
    server:             IpcServer,
}

impl CommandInterface {
    pub fn start(daemon_event_tx: Sender<DaemonEvent>, path: &str) -> Self {
        let server = Self::create_server(path, daemon_event_tx);
        Self {
            server,
        }
    }

    fn create_server(path: &str, daemon_event_tx: mpsc::Sender<DaemonEvent>)
                     -> talpid_ipc::IpcServer{
        let rpc = CommandApiImpl {
            daemon_event_tx: Mutex::new(daemon_event_tx),
        };
        let mut io = IoHandler::new();
        io.extend_with(rpc.to_delegate());

        let ipc_path = if cfg!(windows) {
            format!(r"\\.\pipe\{}", path)
        } else {
            path.to_owned()
        };
        let server = talpid_ipc::IpcServer::start(io.into(), &ipc_path).unwrap();
        return server;
    }
}

struct CommandApiImpl {
    daemon_event_tx: Mutex<mpsc::Sender<DaemonEvent>>,
}

impl CommandApiTrait for CommandApiImpl {
    fn tunnel_connect(&self) -> Result<CommandResponse, Error> {
        let (tx, rx) = mpsc::channel();
        self.daemon_event_tx.lock().unwrap()
            .send(DaemonEvent::IpcCommand(IpcCommand::TunnelConnect(tx))).unwrap();

        if let Ok(response) = rx.recv_timeout(Duration::from_secs(5)) {
            return Ok(response);
        }
        else {
            return Ok(CommandResponse::exec_timeout())
        }
    }

    fn tunnel_disconnect(&self) -> Result<CommandResponse, Error> {
        let (tx, rx) = mpsc::channel();
        self.daemon_event_tx.lock().unwrap()
            .send(DaemonEvent::IpcCommand(IpcCommand::TunnelDisConnect(tx))).unwrap();
        if let Ok(response) = rx.recv_timeout(Duration::from_secs(5)) {
            return Ok(response);
        }
        else {
            return Ok(CommandResponse::exec_timeout())
        }
    }

    fn tunnel_status(&self) -> Result<CommandResponse, Error> {
        let (tx, rx) = mpsc::channel();
        self.daemon_event_tx.lock().unwrap()
            .send(DaemonEvent::IpcCommand(IpcCommand::TunnelStatus(tx))).unwrap();
        if let Ok(response) = rx.recv_timeout(Duration::from_secs(5)) {
            return Ok(response);
        }
        else {
            return Ok(CommandResponse::exec_timeout())
        }
    }

    fn rpc_status(&self) -> Result<CommandResponse, Error> {
        let (tx, rx) = mpsc::channel();
        self.daemon_event_tx.lock().unwrap()
            .send(DaemonEvent::IpcCommand(IpcCommand::RpcStatus(tx))).unwrap();
        if let Ok(response) = rx.recv_timeout(Duration::from_secs(5)) {
            return Ok(response);
        }
        else {
            return Ok(CommandResponse::exec_timeout())
        }
    }

    fn group_info(&self, group_id: String) -> Result<CommandResponse, Error> {
        let (tx, rx) = mpsc::channel();
        self.daemon_event_tx.lock().unwrap()
            .send(DaemonEvent::IpcCommand(IpcCommand::GroupInfo(tx, group_id))).unwrap();
        if let Ok(response) = rx.recv_timeout(Duration::from_secs(5)) {
            return Ok(response);
        }
        else {
            return Ok(CommandResponse::exec_timeout())
        }
    }
}

fn create_client(ipc_path: String) -> jsonrpc_client_core::ClientHandle {
    use std::thread;
    let (tx, rx) = oneshot::channel();

    thread::spawn(move || {
        let (client, client_handle) =
            jsonrpc_client_ipc::IpcTransport::new(&ipc_path, &tokio::reactor::Handle::default())
                .expect("failed to construct a transport")
                .into_client();
        tx.send(client_handle).unwrap();
        client.wait().unwrap();
    });

    rx.wait().expect("Failed to construct a valid client")
}