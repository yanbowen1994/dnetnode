use std::io::Write;
use std::io::Read;
use std::time::Duration;
use std::str::FromStr;
use std::net::{SocketAddrV4, Ipv4Addr, Shutdown, SocketAddr, TcpStream, IpAddr};
use std::collections::HashMap;

use socket2::{Domain, Protocol, Type};

#[derive(err_derive::Error, Debug)]
#[allow(non_camel_case_types)]
pub enum Error {
    #[error(display = "tinc_socket_connect")]
    tinc_socket_connect(String),

    #[error(display = "send_line")]
    send_line,

    #[error(display = "pid_path")]
    pid_path,

    #[error(display = "parse_pid_file")]
    parse_pid_file,

    #[error(display = "reload")]
    reload,

    #[error(display = "restart")]
    restart,

    #[error(display = "dump_nodes")]
    dump_nodes,

    #[error(display = "dump_edges")]
    dump_edges,

    #[error(display = "dump_subnets")]
    dump_subnets,

    #[error(display = "dump_connections")]
    dump_connections,

    #[error(display = "dump_graph")]
    dump_graph,

    #[error(display = "purge")]
    purge,

    #[error(display = "set_debug")]
    set_debug,

    #[error(display = "retry")]
    retry,

    #[error(display = "connect")]
    connect,

    #[error(display = "disconnect")]
    disconnect,

    #[error(display = "dump_traffic")]
    dump_traffic,

    #[error(display = "pcap")]
    pcap,

    #[error(display = "log_level")]
    log_level,

    #[error(display = "dump_group")]
    dump_group,

    #[error(display = "del_group")]
    del_group,

    #[error(display = "del_group_node")]
    del_group_node,

    #[error(display = "subscribe")]
    subscribe,

    #[error(display = "recv_from_subscribe")]
    recv_from_subscribe,

    #[error(display = "parse_source_subnet")]
    parse_source_subnet,

    #[error(display = "parse_source_node")]
    parse_source_node,

    #[error(display = "parse_source_connection")]
    parse_source_connection,

    #[error(display = "parse_source_edge")]
    parse_source_edge,

}

pub type Result<T> = std::result::Result<T, Error>;

use crate::TincTools;

#[allow(dead_code)]
#[repr(i8)]
pub enum  Request {
    All                      = -1,
    Id                       = 0,
    MetaKey                  = 1,
    Challenge                = 2,
    ChalReply                = 3,
    Ack                      = 4,
    Status                   = 5,
    Error                    = 6,
    Termreq                  = 7,
    Ping                     = 8,
    Pong                     = 9,
    AddSubnet                = 10,
    DelSubnet                = 11,
    AddEdge                  = 12,
    DelEdge                  = 13,
    KeyChanged               = 14,
    ReqKey                   = 15,
    AnsKey                   = 16,
    Packet                   = 17,
    Control                  = 18,
    ReqPubkey                = 19,
    AnsPubkey                = 20,
    SptpsPacket              = 21,
    UdpInfo                  = 22,
    MtuInfo                  = 23,
    Last                     = 24,
}

#[allow(dead_code)]
#[repr(i8)]
pub enum RequestType {
    ReqInvalid               = -1,
    ReqStop                  = 0,
    ReqReload                = 1,
    ReqRestart               = 2,
    ReqDumpNodes             = 3,
    ReqDumpEdges             = 4,
    ReqDumpSubnets           = 5,
    ReqDumpConnections       = 6,
    ReqDumpGraph             = 7,
    ReqPurge                 = 8,
    ReqSetDebug              = 9,
    ReqRetry                 = 10,
    ReqConnect               = 11,
    ReqDisconnect            = 12,
    ReqDumpTraffic           = 13,
    ReqPcap                  = 14,
    ReqLog                   = 15,
    ReqDumpEvents            = 16,
    ReqDumpGroups            = 17,
    ReqGroup                 = 18,
    SubScribe                = 19,
}

pub struct TincStream {
    stream: TcpStream,
}
impl TincStream {
    pub fn new(pid_path: &str) -> Result<Self> {
        let (control_cookie, tinc_ip, tinc_port) =
            Self::parse_control_cookie(pid_path)?;
        let buf = format!("{} ^{} {}\n", 0, control_cookie, 17);
        let addr = SocketAddr::from_str(&(tinc_ip.clone() + ":" + &tinc_port))
            .map_err(|_|Error::tinc_socket_connect(tinc_ip + ":" + &tinc_port))?;

        let stream = TcpStream::connect(&addr)
            .map_err(|_| Error::tinc_socket_connect("connect".to_string()))?;
        let _ = stream.set_read_timeout(Some(Duration::from_millis(400)));

        let mut tinc_stream = TincStream{stream};
        tinc_stream.send_line(buf.as_bytes())?;
        let _res = tinc_stream.recv()?;
        return Ok(tinc_stream);
    }

    fn send_line(&mut self, buf: &[u8]) -> Result<()> {
        self.stream.write(buf)
            .map_err(|_|Error::send_line)?;
        return Ok(());
    }

    fn parse_control_cookie(path: &str) -> Result<(String, String, String)> {
        let contents = TincTools::get_tinc_pid_file_all_string(path)
            .ok_or(Error::pid_path)?;

        let iter: Vec<&str> = contents.split_whitespace().collect();
        if iter.len() < 3 {
            error!("Tinc pid file, not find port setting. Maybe tinc tcp port never be set");
            return Err(Error::parse_pid_file);
        }

        let control_cookie = iter[1];

        let mut tinc_ip: &str = "";
        let mut _tinc_port: &str = "";

        if iter.len() >= 5 {
            if iter[2] == "::1" {
                tinc_ip = "127.0.0.1"
            }
            else {
                tinc_ip = iter[2];
            }
            _tinc_port = iter[4];
        }
        return Ok((control_cookie.to_string(), tinc_ip.to_string(), _tinc_port.to_string()));
    }

    pub fn connect_test(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqInvalid as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqInvalid as i8) {
            return Ok(());
        }
        return Ok(());
    }

    pub fn stop(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqStop as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqStop as i8) {
            return Ok(());
        }
        return Ok(());
    }

    pub fn reload(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqReload as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqReload as i8) {
            return Ok(());
        }
        return Err(Error::reload);
    }

    pub fn restart(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqRestart as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqRestart as i8) {
            return Ok(());
        }
        return Err(Error::restart);
    }

    pub fn dump_nodes(&mut self) -> Result<Vec<SourceNode>> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpNodes as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpNodes as i8) {
            if let Ok(source_node) = SourceNode::from_nodes(&res) {
                return Ok(source_node);
            }
            else {
                error!("dump_nodes parse source tinc node info");
            }
        }
        else {
            error!("dump_nodes check_res");
        }
        return Err(Error::dump_nodes);
    }

    pub fn dump_edges(&mut self) -> Result<Vec<SourceEdge>> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpEdges as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        trace!("{}", res);
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpEdges as i8) {
            if let Ok(source_edge) = SourceEdge::from_edges(&res) {
                return Ok(source_edge);
            }
            else {
                error!("dump_edges parse source tinc edges info");
            }
        }
        else {
            error!("dump_edges check_res");
        }
        return Err(Error::dump_edges);
    }

    pub fn dump_subnets(&mut self) -> Result<Vec<SourceSubnet>> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpSubnets as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpSubnets as i8) {
            if let Ok(source_subnet) = SourceSubnet::from_subnets(&res) {
                return Ok(source_subnet);
            }
        }
        return Err(Error::dump_subnets);
    }

    pub fn dump_connections(&mut self) -> Result<Vec<SourceConnection>> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpConnections as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpConnections as i8) {
            if let Ok(source_connection) = SourceConnection::from_connections(&res) {
                return Ok(source_connection);
            }
        }
        return Err(Error::dump_connections);
    }

    pub fn dump_graph(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpGraph as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpGraph as i8) {
            return Ok(());
        }
        return Err(Error::dump_graph);
    }

    pub fn purge(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqPurge as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqPurge as i8) {
            return Ok(());
        }
        return Err(Error::purge);
    }

    pub fn set_debug(&mut self, debug_level: i8) -> Result<()> {
        let cmd = format!("{} {} {}\n", Request::Control as i8, RequestType::ReqSetDebug as i8, debug_level);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqSetDebug as i8) {
            return Ok(());
        }
        return Err(Error::set_debug);
    }

    pub fn retry(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqRetry as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqRetry as i8) {
            return Ok(());
        }
        return Err(Error::retry);
    }

    pub fn connect(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqConnect as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqConnect as i8) {
            return Ok(());
        }
        return Err(Error::connect);
    }

    pub fn disconnect(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDisconnect as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDisconnect as i8) {
            return Ok(());
        }
        return Err(Error::disconnect);
    }

    pub fn dump_traffic(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpTraffic as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpTraffic as i8) {
            return Ok(());
        }
        return Err(Error::dump_traffic);
    }

    pub fn pcap(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqPcap as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqPcap as i8) {
            return Ok(());
        }
        return Err(Error::pcap);
    }

    pub fn log(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqLog as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqLog as i8) {
            return Ok(());
        }
        return Err(Error::log_level);
    }

    pub fn dump_group(&mut self) -> Result<HashMap<String, Vec<IpAddr>>> {
        let cmd = format!("{} {} all\n",
                          Request::Control as i8,
                          RequestType::ReqDumpGroups as i8,
        );
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpGroups as i8) {
            let group_info = Self::parse_source_group_info(&res);
            return Ok(group_info);
        }
        return Err(Error::dump_group);
    }

    fn parse_source_group_info(source_groups: &str) -> HashMap<String, Vec<IpAddr>> {
        let mut output_node_info: HashMap<String, Vec<IpAddr>> = HashMap::new();
        let nodes: Vec<&str> = source_groups.split(" \n").collect();

        for nodes_str in nodes {
            let item: Vec<&str> = nodes_str
                .split(": ")
                .collect();
            let mut members = vec![];
            if item.len() == 2 {
                let nodes: Vec<&str> = item[1]
                    .split(" ")
                    .collect();

                for i in nodes {
                    let vip: Vec<&str> = i.split("_").collect();
                    if vip.len() == 3 {
                        let vip = match IpAddr::from_str(
                            &format!("10.{}.{}.{}",
                                     vip[0].replace(" ", ""),
                                     vip[1],
                                     vip[2])) {
                            Ok(x) => x,
                            Err(_) => continue,
                        };
                        members.push(vip)
                    }
                }
            }
            members.sort();
            members.dedup();

            let index_node: Vec<&str> = item[0].split_ascii_whitespace().collect();

            if index_node.len() == 3 {
                let team_id = index_node[2].to_owned();
                output_node_info.insert(team_id, members);
            }
        }

        output_node_info
    }

    pub fn del_group(&mut self, group_id: &str) -> Result<()> {
        let cmd = format!("{} {} delvlan {} ...\n",
                          Request::Control as i8,
                          RequestType::ReqGroup as i8,
                          group_id,
        );
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        info!("{:?}", res);
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqGroup as i8) {
            return Ok(());
        }
        return Err(Error::del_group);
    }

    pub fn add_group_node(&mut self,
                          groups: &HashMap<String, Vec<IpAddr>>
    ) -> Result<std::result::Result<(), Vec<String>>> {
        let buf = Self::parse_groups(groups);
        let cmd = format!("{} {} addvlan {} .\n",
                          Request::Control as i8,
                          RequestType::ReqGroup as i8,
                          buf,
        );
        info!("add_group_node success send_line: {:?}", cmd);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if res.contains("18 18 2") {
            error!("add_group_node {:?}", res);
        }
        else {
            info!("add_group_node success res: {:?}", res);
        }
        if let Some(failed_group) = Self::check_group_res(&res, Request::Control as i8, RequestType::ReqGroup as i8) {
            return Ok(Err(failed_group));
        }
        else {
            return Ok(Ok(()))
        }
    }

    fn parse_groups(groups: &HashMap<String, Vec<IpAddr>>) -> String {
        let mut out = String::new();
        for (group_id, nodes) in groups {
            let mut group_buf = String::new();
            if nodes.len() > 0 {
                group_buf += group_id;
                for node in nodes {
                    group_buf = group_buf + "," + &TincTools::get_filename_by_vip(false, &node.to_string());
                }
            }
            group_buf += "#";
            out += &group_buf;
        }
        out
    }

    pub fn del_group_node(&mut self, group_id: &str, node_id: &str) -> Result<()> {
        let buf = group_id.to_string() + "," + node_id;
        let cmd = format!("{} {} deln {} ...\n",
                          Request::Control as i8,
                          RequestType::ReqGroup as i8,
                          buf,
        );
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        info!("{:?}", res);
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqGroup as i8) {
            return Ok(());
        }
        return Err(Error::del_group_node);
    }

    pub fn subscribe(pid_path: &str) -> Result<socket2::Socket> {
        let (control_cookie, tinc_ip, tinc_port) =
            Self::parse_control_cookie(pid_path)?;

        let addr = socket2::SockAddr::from(
            SocketAddrV4::new(
                Ipv4Addr::from_str(&tinc_ip)
                    .map_err(|_|Error::subscribe)?,
                tinc_port.parse::<u16>()
                    .map_err(|_|Error::subscribe)?
            )
        );

        let mut socket = socket2::Socket::new(
            Domain::ipv4(),
            Type::stream(),
            Some(Protocol::tcp())
        ).unwrap();
        if let Ok(_) = socket.connect(&addr) {
            let buf = format!("{} ^{} {}\n", 0, control_cookie, 17);
            socket.set_write_timeout(Some(Duration::from_millis(200)))
                .map_err(|_|Error::subscribe)?;
            socket.write_all(buf.as_bytes())
                .map_err(|_|Error::subscribe)?;

            let cmd = format!("{} {} subscribe true\n",
                             Request::Control as i8,
                             RequestType::SubScribe as i8,
            );

            socket.write_all(cmd.as_bytes())
                .map_err(|_|Error::subscribe)?;
            socket.set_read_timeout(Some(Duration::from_millis(400)))
                .map_err(|_|Error::subscribe)?;
            return Ok(socket);
        }
        let _ = socket.shutdown(Shutdown::Both);
        Err(Error::subscribe)
    }

    pub fn recv_from_subscribe(socket: &socket2::Socket) -> Result<String> {
        let mut buffer: [u8; 2048] = [0; 2048];
        match socket.recv_from(&mut buffer) {
            Ok(_) => return Ok(String::from_utf8(buffer.to_vec())
                .unwrap_or(String::new())
            ),
            Err(_) => return Err(Error::recv_from_subscribe),
        }
    }

    fn recv(&mut self) -> Result<String> {
        let mut output = String::new();
        loop {
            let res = &mut [0; 128];
            let _len = match self.stream.read(res) {
                Ok(x) => x,
                Err(_) => 0,
            };

            if _len == 0 {
                break
            }
            else {
                if let Ok(res) = String::from_utf8(res.to_vec()) {
                    if res.contains("\u{0}") {
                        output += &res.replace("\u{0}", "");
                        break
                    }
                    output += &res;
                }
            }
        }
        Ok(output)
    }

    fn check_res(res: &str, req: i8, req_type: i8) -> bool {
        let iter: Vec<&str> = res.split_whitespace().collect();
        if iter.len() < 2 {
            return false;
        }
        let control: i8 = match iter[0].parse() {
            Ok(x) => x,
            _ => -1,
        };

        let control_type: i8 = match iter[1].parse() {
            Ok(x) => x,
            _ => -1,
        };
        if control == req && control_type == req_type {
            return true;
        }
        false
    }

    // if return None, means Ok.
    // if return vec![], means all group set failed.
    fn check_group_res(res: &str, req: i8, req_type: i8) -> Option<Vec<String>> {
        let iter: Vec<&str> = res.split_whitespace().collect();
        if iter.len() < 2 {
            return Some(vec![]);
        }

        else if iter.len() == 2 {
            let control: i8 = match iter[0].parse() {
                Ok(x) => x,
                _ => -1,
            };

            let control_type: i8 = match iter[1].parse() {
                Ok(x) => x,
                _ => -1,
            };

            if control == req && control_type == req_type {
                return None;
            }
            else {
                return Some(vec![]);
            }
        }
        else if iter.len() == 3 {
                return Some(vec![]);
        }
        else {
            let failed_groups: Vec<&str> = iter[3]
                .split("\n")
                .collect::<Vec<&str>>();
            let failed_groups = failed_groups[0]
                .split(",")
                .collect::<Vec<&str>>()
                .into_iter()
                .map(|group|group.to_string())
                .collect::<Vec<String>>();

            Some(failed_groups)
        }
    }
}

impl Drop for TincStream {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(Shutdown::Both);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceNode {
    pub node:                   String,
    pub id:                     String,
    pub host:                   String,
    pub _a:                     String,
    pub port:                   String,
    pub cipher:                 String,
    pub digest:                 String,
    pub maclength:              String,
    pub compression:            String,
    pub options:                String,
    pub status_int:             String,
    pub nexthop:                String,
    pub via:                    String,
    pub distance:               String,
    pub pmtu:                   String,
    pub minmtu:                 String,
    pub maxmtu:                 String,
    pub last_state_change:      String,
}
impl SourceNode {
    pub fn from(source_str: &str) -> Result<Self> {
        let source_str = source_str.to_string();
        let node_str:Vec<&str> = source_str.split_ascii_whitespace().collect();
        if node_str.len() >= 20 {
            let node: String = node_str[2].to_string();
            let id: String = node_str[3].to_string();
            let host: String = node_str[4].to_string();
            let _a: String = node_str[5].to_string();
            let port: String = node_str[6].to_string();
            let cipher: String = node_str[7].to_string();
            let digest: String = node_str[8].to_string();
            let maclength: String = node_str[9].to_string();
            let compression: String = node_str[10].to_string();
            let options: String = node_str[11].to_string();
            let status_int: String = node_str[12].to_string();
            let nexthop: String = node_str[13].to_string();
            let via: String = node_str[14].to_string();
            let distance: String = node_str[15].to_string();
            let pmtu: String = node_str[16].to_string();
            let minmtu: String = node_str[17].to_string();
            let maxmtu: String = node_str[18].to_string();
            let last_state_change: String = node_str[19].to_string();
            let source_node = SourceNode {
                node,
                id,
                host,
                _a,
                port,
                cipher,
                digest,
                maclength,
                compression,
                options,
                status_int,
                nexthop,
                via,
                distance,
                pmtu,
                minmtu,
                maxmtu,
                last_state_change,
            };
            return Ok(source_node);
        }
        return Err(Error::parse_source_node);
    }

    pub fn from_nodes(source_info: &str) -> Result<Vec<Self>> {
        let source_info = source_info.to_string();
        let nodes_str:Vec<&str> = source_info.split("\n").collect();
        let mut nodes: Vec<SourceNode> = vec![];
        for node_str in nodes_str.clone() {
            if node_str.len() > 0 {
                if let Ok(node) = SourceNode::from(node_str) {
                    nodes.push(node)
                }
            }
        }
        return Ok(nodes);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceEdge {
    pub from:           String,
    pub to:             String,
    pub host:           String,
    pub _a:             String,
    pub port:           String,
    pub local_host:     String,
    pub _b:             String,
    pub local_port:     String,
    pub options:        String,
    pub weight:         String,
//    avg_rtt:        String,
}
impl SourceEdge{
    fn from(source_str: &str) -> Result<Self> {
        let edge_str:Vec<&str> = source_str.split(" ").collect();
        if edge_str.len() >= 12 {
            let from = edge_str[2].to_string();
            let to = edge_str[3].to_string();
            let host = edge_str[4].to_string();
            let _a = edge_str[5].to_string();
            let port = edge_str[6].to_string();
            let local_host = edge_str[7].to_string();
            let _b = edge_str[8].to_string();
            let local_port = edge_str[9].to_string();
            let options = edge_str[10].to_string();
            let weight = edge_str[11].to_string();
            return Ok(SourceEdge {
                from,
                to,
                host,
                _a,
                port,
                local_host,
                _b,
                local_port,
                options,
                weight,
            });
        }
        return Err(Error::parse_source_edge);
    }

    fn from_edges(source_info: &str) -> Result<Vec<Self>> {
        let source_info = source_info.to_string();
        let edges_str:Vec<&str> = source_info.split("\n").collect();
        let mut edges: Vec<SourceEdge> = vec![];
        for edge_str in edges_str.clone() {
            if edge_str.len() > 0 {
                if let Ok(edge) = SourceEdge::from(edge_str) {
                    edges.push(edge)
                }
            }
        }
        return Ok(edges);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceSubnet {
    pub name:       String,
    pub addr:       String,
}
impl SourceSubnet{
    fn from(source_str: &str) -> Result<Self> {
        let subnet_str:Vec<&str> = source_str.split(" ").collect();
        if subnet_str.len() >= 4 {
            let addr = subnet_str[2].to_string();
            let name = subnet_str[3].to_string();
            return Ok(SourceSubnet {
                name,
                addr,
            });
        }
        return Err(Error::parse_source_subnet);
    }

    fn from_subnets(source_info: &str) -> Result<Vec<Self>> {
        let source_info = source_info.to_string();
        let subnets_str:Vec<&str> = source_info.split("\n").collect();
        let mut subnets: Vec<Self> = vec![];
        for subnet_str in subnets_str.clone() {
            if subnet_str.len() > 0 {
                if let Ok(subnet) = Self::from(subnet_str) {
                    subnets.push(subnet)
                }
            }
        }
        return Ok(subnets);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceConnection {
    pub node:           String,
    pub host:           String,
    pub _a:             String,
    pub port:           String,
    pub options:        String,
    pub socket:         String,
    pub status_int:     String,
}
impl SourceConnection{
    fn from(source_str: &str) -> Result<Self> {
        let connection_str:Vec<&str> = source_str.split(" ").collect();
        if connection_str.len() >= 9 {
            let node = connection_str[2].to_string();
            let host = connection_str[3].to_string();
            let _a = connection_str[4].to_string();
            let port = connection_str[5].to_string();
            let options = connection_str[6].to_string();
            let socket = connection_str[7].to_string();
            let status_int = connection_str[8].to_string();
            return Ok(SourceConnection {
                node,
                host,
                _a,
                port,
                options,
                socket,
                status_int,
            });
        }
        return Err(Error::parse_source_connection);
    }

    fn from_connections(source_info: &str) -> Result<Vec<Self>> {
        let source_info = source_info.to_string();
        let connections_str:Vec<&str> = source_info.split("\n").collect();
        let mut connections: Vec<Self> = vec![];
        for connection_str in connections_str.clone() {
            if connection_str.len() > 0 {
                if let Ok(connection) = Self::from(connection_str) {
                    connections.push(connection)
                }
            }
        }
        return Ok(connections);
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::net::IpAddr;
    use std::str::FromStr;
    use crate::tinc_tcp_stream::TincStream;

    #[test]
    fn test_add_group_node() {
        let members = vec![IpAddr::from_str("10.1.1.1").unwrap(),
                           IpAddr::from_str("10.1.1.2").unwrap()];
        let mut groups = HashMap::new();
        groups.insert("123".to_string(), members.clone());
        groups.insert("456".to_string(), members);

        let mut tinc_stream = TincStream::new("/opt/dnet/tinc/tinc.pid")
            .expect("Tinc socket connect failed.");
        println!("{:?}", tinc_stream.add_group_node(&groups));
    }

    #[test]
    fn test_del_group_node() {
        let mut tinc_stream = TincStream::new("/opt/dnet/tinc/tinc.pid")
            .expect("Tinc socket connect failed.");
        println!("{:?}", tinc_stream.del_group_node("123", "1_1_1"));
    }

    #[test]
    fn test_del_group() {
        let mut tinc_stream = TincStream::new("/opt/dnet/tinc/tinc.pid")
            .expect("Tinc socket connect failed.");
        println!("{:?}", tinc_stream.del_group("456"));
    }
}