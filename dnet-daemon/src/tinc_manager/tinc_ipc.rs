use jsonrpc_core::{
    futures::{
        future,
        sync::{self, oneshot::Sender as OneshotSender},
        Future,
    },
    Error, MetaIoHandler, Metadata, IoHandler,
};

use tinc_plugin::listener;
use std::sync::mpsc::Sender;
use talpid_ipc;

pub fn start(tinc_event_tx: Sender<listener::HostStatusChange>) -> std::result::Result<talpid_ipc::IpcServer, u32> {
    let ipc_path = if cfg!(windows) {
        format!("//./pipe/tinc-event")
    } else {
        format!("/tmp/tinc-event.socket")
    };
    let rpc = TincEventApiImpl { tinc_event_tx };
    let mut io = IoHandler::new();
    io.extend_with(rpc.to_delegate());
    let meta_io: MetaIoHandler<()> = MetaIoHandler::from(io);
    talpid_ipc::IpcServer::start(meta_io, &ipc_path)
}

build_rpc_trait! {
    pub trait TincEventApi {
        #[rpc(name = "tinc_event")]
        fn tinc_event(&self, listener::HostStatusChange)
            -> Result<(), Error>;
    }
}

struct TincEventApiImpl {
    tinc_event_tx: Sender<listener::HostStatusChange>,
}

impl TincEventApi for TincEventApiImpl {
    fn tinc_event(
        &self,
        event: listener::HostStatusChange) -> Result<(), u32> {
        log::trace!("OpenVPN event {:?}", event);
        Ok(())
    }
}