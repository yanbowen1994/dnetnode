use std::process::Command;

use super::types::IptablesRule;

pub fn iptabels_create_chain(table: &str, chain: &str) {
    let _ = Command::new("iptables").args(vec![
        "-t",
        table,
        "-N",
        chain,
    ]).spawn();
}

fn parse_on_rule(rule: &str) -> Option<IptablesRule> {
    let segs = rule.split_ascii_whitespace().collect::<Vec<&str>>();
    if segs.len() < 9 {
        return None;
    }

    let iptables_rule = IptablesRule {
        in_:        segs[5].to_owned(),
        out:        segs[6].to_owned(),
        src:        segs[7].to_owned(),
        dst:        segs[8].to_owned(),
        target:     segs[2].to_owned(),
    };
    
    Some(iptables_rule)
}



pub fn iptables_append_rule(table: &str, chain: &str, rule_str: &str)
{
    let _ = Command::new("iptables").args(vec![
        "-t",
        table,
        "-A",
        chain,
        rule_str,
    ]).spawn();
}


pub fn iptables_insert_rule(table: &str, chain: &str, rule_str: &str, position: i32) {
    if position != 0 {
        let _ = Command::new("iptables").args(vec![
            "-t",
            table,
            "-I",
            chain,
            &format!("{}", position),
            rule_str,
        ]).spawn();
    }
    else {
        let _ = Command::new("iptables").args(vec![
            "-t",
            table,
            "-I",
            chain,
            rule_str,
        ]).spawn();
    }
}

pub fn iptables_delete_rule(table: &str, chain: &str, rule_str: &str) {
    let _ = Command::new("iptables").args(vec![
        "-t",
        table,
        "-D",
        chain,
        rule_str,
    ]).spawn();
}

