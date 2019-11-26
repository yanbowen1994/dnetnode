use std::io::Write;
use std::process::Command;
use std::os::unix::fs::PermissionsExt;

use sandbox::firewall::{imp::{iptables_insert_rule, iptables_find_rule}, types::IptablesRule};

pub fn start_firewall(port: u16) {
    firewall_script_write(port);
    firewall_script_start();
    vpn_tunnel_firewall();
}

pub fn stop_firewall(port: u16) {
    firewall_script_write(port);
    firewall_script_stop();
}

fn firewall_script_write(port: u16) {
    let port = format!("{}", port);

    let buf = "#! /bin/sh\n\
    if [ \"$1\" == \"start\" ]; then\n\
        \t/usr/sbin/iptables -I INPUT -i dnet -j ACCEPT\n\
        \t/usr/sbin/iptables -I OUTPUT -o dnet -j ACCEPT\n\
        \t/usr/sbin/iptables -I INPUT -i brwan -p udp --dport " + &port + " -j ACCEPT\n\
        \t/usr/sbin/iptables -I INPUT -i ppp0 -p udp --dport " + &port + " -j ACCEPT\n\
        \t/usr/sbin/iptables -I INPUT -i brwan -p tcp --dport " + &port + " -j ACCEPT\n\
        \t/usr/sbin/iptables -I INPUT -i ppp0 -p tcp --dport " + &port + " -j ACCEPT\n\
        \t/usr/sbin/iptables -I FORWARD -i dnet -j ACCEPT\n\
        \t/usr/sbin/iptables -I FORWARD -o dnet -j ACCEPT\n\
        \t/usr/sbin/iptables -t nat -I POSTROUTING -o dnet -j MASQUERADE\n\
    fi;\n\
    if [ \"$1\" == \"stop\" ]; then\n\
        \t/usr/sbin/iptables -D INPUT -i dnet -j ACCEPT\n\
        \t/usr/sbin/iptables -D OUTPUT -o dnet -j ACCEPT\n\
        \t/usr/sbin/iptables -D INPUT -i brwan -p udp --dport " + &port + " -j ACCEPT\n\
        \t/usr/sbin/iptables -D INPUT -i ppp0 -p udp --dport " + &port + " -j ACCEPT\n\
        \t/usr/sbin/iptables -D INPUT -i brwan -p tcp --dport " + &port + " -j ACCEPT\n\
        \t/usr/sbin/iptables -D INPUT -i ppp0 -p tcp --dport " + &port + " -j ACCEPT\n\
        \t/usr/sbin/iptables -D FORWARD -i dnet -j ACCEPT\n\
        \t/usr/sbin/iptables -D FORWARD -o dnet -j ACCEPT\n\
        \t/usr/sbin/iptables -t nat -D POSTROUTING -o dnet -j MASQUERADE\n\
    fi;\n\
    ";
    let path = "/etc/scripts/firewall/vppn.rule";
    if let Ok(mut file) = std::fs::File::create(&path) {
        let _ = file.write_all(buf.as_bytes());
    }
    let _ = std::fs::set_permissions(&path, PermissionsExt::from_mode(0o755));
}

fn firewall_script_start() {
    if let Ok(mut child) = Command::new("/etc/scripts/firewall/vppn.rule")
        .arg("start").spawn() {
        let _ = child.wait();
    }
}

fn firewall_script_stop() {
    if let Ok(mut child) = Command::new("/etc/scripts/firewall/vppn.rule")
        .arg("stop").spawn() {
        let _ = child.wait();
    }
}

fn vpn_tunnel_firewall() {
    let tun_buf = "dnet";

    /* add rule in nat POSTROUTING */
    let rule = IptablesRule::new(
        "nat",
        "POSTROUTING",
        "",
        "dnet",
        "",
        "",
        "MASQUERADE"
    );
    if !iptables_find_rule(&rule) {
        let rule_str = format!("-o {} -j MASQUERADE", tun_buf);
        iptables_insert_rule("nat", "POSTROUTING", &rule_str, 0);
    }

    /* add rule in filter FORWARD */
    let rule = IptablesRule::new(
        "filter",
        "FORWARD",
        "br0",
        tun_buf,
        "",
        "",
        "ACCEPT"
    );
    if !iptables_find_rule(&rule) {
        let rule_str = format!("-i br0 -o {} -j ACCEPT", tun_buf);
        iptables_insert_rule("filter",
                             "FORWARD",
                             &rule_str,
                             0);
    }

    /* add rule in filter FORWARD */
    let rule = IptablesRule::new(
        "filter",
        "FORWARD",
        tun_buf,
        "br0",
        "",
        "",
        "ACCEPT"
    );
    if !iptables_find_rule(&rule) {
        let rule_str = format!("-i {} -o br0 -j ACCEPT", tun_buf);
        iptables_insert_rule("filter",
                             "FORWARD",
                             &rule_str,
                             0);
    }

    /* add rule in filter INPUT */
    let rule = IptablesRule::new(
        "filter",
        "INPUT",
        tun_buf,
        "",
        "",
        "",
        "ACCEPT");
    if iptables_find_rule(&rule) {
        let rule_str = format!("-i {} -j ACCEPT", tun_buf);
        iptables_insert_rule("filter",
                             "INPUT",
                             &rule_str,
                             0);
    }

    /* add rule in filter OUTPUT */
    let rule = IptablesRule::new(
        "filter",
        "OUTPUT",
        "",
        tun_buf,
        "",
        "",
        "ACCEPT");
    if !iptables_find_rule(&rule) {
        let rule_str = format!("-o {} -j ACCEPT", tun_buf);
        iptables_insert_rule("filter", "OUTPUT", &rule_str, 0);
    }
}