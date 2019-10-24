extern crate sandbox;

use sandbox::firewall::imp::*;
use sandbox::firewall::types::IptablesRule;

fn main() {
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