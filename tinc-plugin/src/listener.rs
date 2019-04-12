use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io::{Error, ErrorKind, Result, Read};

#[derive(Debug)]
pub enum EventType {
    Up,
    Down,
    HostUp(String),
    HostDown(String),
}

fn handle_client(stream: &mut TcpStream) -> Result<EventType> {
    let mut res = &mut[0; 128];
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

pub fn spawn() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 50070));
    let listener = TcpListener::bind(&addr).expect("");
    for res_stream in listener.incoming() {
        if let Ok(mut stream) = res_stream {
            if let Ok(event) = handle_client(&mut stream) {
                println!("{:?}", event);
            }
        }
    }
}