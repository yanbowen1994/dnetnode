use tinc_plugin::tinc_tcp_stream::{TincStream, Result, Error};

pub fn tinc_connections(pid_path: &str) -> Result<(u32, u32, u32)> {
    if !std::path::Path::new(pid_path).is_file() {
        return Err(Error::pid_path);
    }

    let mut tinc_stream = TincStream::new(pid_path)?;
    let connections = tinc_stream.dump_connections()?;
    let edges = tinc_stream.dump_edges()?;
    let nodes = tinc_stream.dump_nodes()?;
    return Ok((connections.len() as u32, edges.len() as u32, nodes.len() as u32));
}