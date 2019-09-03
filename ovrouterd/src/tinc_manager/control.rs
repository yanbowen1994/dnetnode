use crate::tinc_plugin::{TincStream};

pub fn tinc_connections(pid_path: &str) -> ::std::io::Result<(u32, u32, u32)> {
    let mut tinc_stream = TincStream::new(pid_path)?;
    let connections = tinc_stream.dump_connections()?;
    let edges = tinc_stream.dump_edges()?;
    let nodes = tinc_stream.dump_nodes()?;
    return Ok((connections.len() as u32, edges.len() as u32, nodes.len() as u32));
}