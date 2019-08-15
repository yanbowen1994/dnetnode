use std::net::TcpStream;
use std::io::Write;
use std::net::SocketAddr;
use std::fs::File;
use std::io::{Error, ErrorKind, Result, Read};

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

#[repr(i8)]
pub enum RequestType {
    ReqInvalid              = -1,
    ReqStop                 = 0,
    ReqReload               = 1,
    ReqRestart              = 2,
    ReqDumpNodes            = 3,
    ReqDumpEdges            = 4,
    ReqDumpSubnets          = 5,
    ReqDumpConnections      = 6,
    ReqDumpGraph            = 7,
    ReqPurge                = 8,
    ReqSetDebug             = 9,
    ReqRetry                = 10,
    ReqConnect              = 11,
    ReqDisconnect           = 12,
    ReqDumpTraffic          = 13,
    ReqPcap                 = 14,
    ReqLog                  = 15,
}

pub struct TincStream {
    stream: TcpStream,
}
impl TincStream {
    pub fn new(pid_path: &str) -> Result<Self> {
        let control_cookie = Self::parse_control_cookie(pid_path)?;
        let buf = format!("{} ^{} {}\n", 0, control_cookie, 17);
        let addr = SocketAddr::from(([127, 0, 0, 1], 50069));
        let stream = TcpStream::connect(&addr)?;
        let mut tinc_stream = TincStream{stream};
        tinc_stream.send_line(buf.as_bytes())?;
        let _res = tinc_stream.recv()?;
        return Ok(tinc_stream);
    }

    fn send_line(&mut self, buf: &[u8]) -> Result<()> {
        self.stream.write(buf)?;
        return Ok(());
    }

    fn parse_control_cookie(path: &str) -> Result<String> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let iter: Vec<&str> = contents.split_whitespace().collect();
        let control_cookie = iter[1];
        return Ok(control_cookie.to_string());
    }

    pub fn stop(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqStop as i8);
        self.send_line(cmd.as_bytes())?;
        return Ok(());
    }

    pub fn reload(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqReload as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqReload as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Reload failed."));
    }

    pub fn restart(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqRestart as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqReload as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Restart failed."));
    }

    pub fn dump_nodes(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpNodes as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpNodes as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Dump nodes failed."));
    }

    pub fn dump_subnets(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpSubnets as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpSubnets as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Dump subnets failed."));
    }

    pub fn dump_connections(&mut self) -> Result<String> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpConnections as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpConnections as i8) {
            return Ok(res);
        }
        return Err(Error::new(ErrorKind::InvalidData, "Dump connections failed."));
    }

    pub fn dump_graph(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpGraph as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpGraph as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Dump graph failed."));
    }

    pub fn purge(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqPurge as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqPurge as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Purge failed."));
    }

    pub fn set_debug(&mut self, debug_level: i8) -> Result<()> {
        let cmd = format!("{} {} {}\n", Request::Control as i8, RequestType::ReqSetDebug as i8, debug_level);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqSetDebug as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Set debug failed."));
    }

    pub fn retry(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqRetry as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqRetry as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Retry failed."));
    }

    pub fn connect(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqConnect as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqConnect as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Connect failed."));
    }

    pub fn disconnect(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDisconnect as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDisconnect as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Disconnect failed."));
    }

    pub fn dump_traffic(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqDumpTraffic as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqDumpTraffic as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Dump traffic failed."));
    }

    pub fn pcap(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqPcap as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqPcap as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Pcap failed."));
    }

    pub fn log(&mut self) -> Result<()> {
        let cmd = format!("{} {}\n", Request::Control as i8, RequestType::ReqLog as i8);
        self.send_line(cmd.as_bytes())?;
        let res = self.recv()?;
        if Self::check_res(&res, Request::Control as i8, RequestType::ReqLog as i8) {
            return Ok(());
        }
        return Err(Error::new(ErrorKind::InvalidData, "Log failed."));
    }

    fn recv(&mut self) -> Result<String> {
        let res = &mut[0; 64];
        let _len = self.stream.read(res)?;
        Ok(String::from_utf8_lossy(res).to_string())
    }

    fn check_res(res: &str, req: i8, req_type: i8) -> bool{
        let iter: Vec<&str> = res.split_whitespace().collect();
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
}