use std::net::{SocketAddr, TcpListener, TcpStream, Shutdown};
use std::io::{Error, ErrorKind, Result, Read};
use std::sync::mpsc;
use std::thread;

//use serde::{de::DeserializeOwned, Serialize};
//#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum EventType {
    Up,
    Down,
    HostUp(String),
    HostDown(String),
}

fn handle_client(stream: &mut TcpStream) -> Result<EventType> {
    let res = &mut[0; 128];
    stream.read(res)?;
    let event_str = String::from_utf8_lossy(res).to_string().replace("\u{0}", "");
    let event_vec: Vec<&str> = event_str.split_whitespace().collect();

    let mut out = EventType::Down;
    match event_vec[0] {
        _ if event_vec[0] == "Up" => out = EventType::Up,
        _ if event_vec[0] == "Down" => out = EventType::Down,
        _ if event_vec[0] == "HostUp" => {
            if event_vec.len() > 1 {
                out = EventType::HostUp(event_vec[1].to_string())
            }
        },
        _ if event_vec[0] == "HostDown" => {
            if event_vec.len() > 1 {
                out = EventType::HostDown(event_vec[1].to_string())
            }
        },
        _ => (return Err(Error::new(ErrorKind::InvalidData, "")))
    };
    return Ok(out);
}

fn listen(addr: SocketAddr, tinc_event_tx: mpsc::Sender<EventType>) {
    let listener = TcpListener::bind(&addr).expect("");
    for res_stream in listener.incoming() {
        if let Ok(mut stream) = res_stream {
            if let Ok(event) = handle_client(&mut stream) {
                log::info!("Tinc event {:?}", event);
                let _ = tinc_event_tx.send(event.clone());
                if event == EventType::Down {
                    let _ = stream.shutdown(Shutdown::Both);
                    break;
                }
            }
        }
    }
}

pub fn spawn() -> mpsc::Receiver<EventType> {
    let (tinc_event_tx, tinc_event_rx) = mpsc::channel();
    let addr = SocketAddr::from(([127, 0, 0, 1], 50070));
    thread::spawn(move ||listen(addr, tinc_event_tx));
    tinc_event_rx
}
