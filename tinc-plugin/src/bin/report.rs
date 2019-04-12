use std::net::{SocketAddr, TcpStream};
use std::io::Write;
use std::env;

fn help() {
    let buf = "\r
    USAGE:\r
          mullvad <FLAGS>\r
    FLAGS:\r
        -h,      Prints help information\r
        -u,      Tinc Up\r
        -d,      Tinc Down\r
        -hu <hostname>,     Host Up\r
        -hd <hostname>,     Host Down";
    println!("{}", buf);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let mut buf = String::new();
        match args[1] {
            _ if args[1] == "-u" => buf = format!("Up"),
            _ if args[1] == "-d" => buf = format!("Down"),
            _ if args[1] == "-hu" => {
                if args.len() > 2 {
                    buf = format!("HostUp {}", args[2])
                }
            },
            _ if args[1] == "-hd" => {
                if args.len() > 2 {
                    buf = format!("HostDown {}", args[2])
                }
            },
            _ => ()
        }
        if buf.len() > 0 {
            let addr = SocketAddr::from(([127, 0, 0, 1], 50070));
            let mut stream = TcpStream::connect(&addr).expect("Tinc Monitor not exist");
            stream.write(buf.as_bytes()).expect("Tinc Monitor not exist");
            return;
        }
    }
    help();
}