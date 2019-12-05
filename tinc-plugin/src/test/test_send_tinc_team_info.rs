mod create_test_env;

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use tinc_plugin::{TincOperator, TincStream};
use std::time::Duration;

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

fn test_del_group_node() {
    let mut tinc_stream = TincStream::new("/opt/dnet/tinc/tinc.pid")
        .expect("Tinc socket connect failed.");
    println!("{:?}", tinc_stream.del_group_node("123", "1_1_1"));
}

fn test_del_group() {
    let mut tinc_stream = TincStream::new("/opt/dnet/tinc/tinc.pid")
        .expect("Tinc socket connect failed.");
    println!("{:?}", tinc_stream.del_group("456"));
}

fn dump_group() {
    let mut tinc_stream = TincStream::new("/opt/dnet/tinc/tinc.pid")
        .expect("Tinc socket connect failed.");
    let team_info = tinc_stream.dump_group().unwrap();
    if *team_info.get("123").unwrap() != vec![Ipv4Addr::new(10, 1, 1, 2)] {
        panic!("team info failed.")
    }
    println!("{:?}", team_info);
}

fn main() {
    create_test_env::create_test_env();
//    let tinc = TincOperator::mut_instance();
//
//    let _ = tinc.stop_tinc();
//    tinc.start_tinc().expect("start tinc");

    std::thread::sleep(Duration::from_secs(1));

    loop {
        test_add_group_node();

        test_del_group_node();

        test_del_group();

        dump_group();

        std::thread::sleep(Duration::from_secs(1));
    }

//    tinc.stop_tinc().expect("stop tinc");
}