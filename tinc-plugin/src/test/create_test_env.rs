use std::process::Command;
use std::str::FromStr;
use std::net::IpAddr;
use tinc_plugin::{TincTools, TincOperator, TincSettings, TincRunMode, TincInfo};

pub fn create_test_env() {
    let _ = Command::new("rm")
        .args(vec!["-rf", "/opt/dnet/tinc/hosts"])
        .spawn().unwrap().wait();

    let tinc_home = "/opt/dnet/tinc/".to_owned();
    let tinc_settings = TincSettings {
        tinc_home,
        mode:                               TincRunMode::Center,
        port:                               50069,
        tinc_memory_limit:                  85.0,
        tinc_allowed_out_memory_times:      0,
        tinc_allowed_tcp_failed_times:      0,
        tinc_check_frequency:               0,
        external_boot:                      false,
    };
    TincOperator::new(tinc_settings);

    let (_, pubkey) = TincTools::create_key_pair().unwrap();
    let tinc_info = TincInfo::new(
        Some(IpAddr::from_str("192.168.1.1").unwrap()),
        50069,
        IpAddr::from_str("10.253.1.1").unwrap(),
        &pubkey,
        TincRunMode::Center,
        vec![],
    );

    let tinc = TincOperator::mut_instance();
    tinc.create_tinc_dirs().unwrap();
    tinc.set_info_to_local(&tinc_info).unwrap();

    for i in 1..11 {
        let (_, pubkey) = TincTools::create_key_pair().unwrap();
        let vip = IpAddr::from_str(&("10.1.1.".to_string() + &format!("{}", i)))
            .unwrap();
        tinc.set_hosts(None, vip, &pubkey).unwrap();
    }
}