use std::sync::mpsc;
use std::time::Duration;

use dnet_types::response::Response;
use crate::daemon::{TunnelCommand, DaemonEvent};

pub fn send_tunnel_connect(
    tunnel_command_tx:  mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>,
    daemon_event_tx:    mpsc::Sender<DaemonEvent>,
) -> Response {
    let (res_tx, res_rx) = mpsc::channel::<Response>();
    let _ = tunnel_command_tx.send((TunnelCommand::Connect, res_tx));
    if let Ok(response) = res_rx.recv_timeout(
        Duration::from_secs(3))
    {
        if response.code == 200 {
            let _ = daemon_event_tx.send(DaemonEvent::TunnelConnected);
            return Response::success();
        } else {
            error!("Tunnel connect failed. error: {:?}", response.msg);
            return response;
        }
    }
    else {
        let response = Response::exec_timeout();
        return response;
    }
}

pub fn send_tunnel_disconnect(
    tunnel_command_tx:  mpsc::Sender<(TunnelCommand, mpsc::Sender<Response>)>,
    daemon_event_tx:    mpsc::Sender<DaemonEvent>,
) -> Response {
    let (res_tx, res_rx) = mpsc::channel::<Response>();
    let _ = tunnel_command_tx.send((TunnelCommand::Disconnect, res_tx));
    let res = res_rx.recv_timeout(Duration::from_secs(5))
        .map(|res|{
            if res.code == 200 {
                let _ = daemon_event_tx.send(DaemonEvent::TunnelDisconnected);
            }
            else {
                error!("Tunnel disconnect failed. error: {:?}", res.msg);
            }
            res
        })
        .map_err(|_| {
            error!("Tunnel disconnect failed. error: Respones recv timeout.")
        })
        .unwrap_or(Response::exec_timeout());
    res
}