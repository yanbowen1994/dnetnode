use std::process::Command;

use super::types::IptablesRule;

pub fn parse_iptables_table_chain(table: &str, chain: &str) -> Vec<IptablesRule> {
    if let Ok(output) = Command::new("iptables")
        .args(vec![
            "-t",
            table,
            "-L",
            chain,
            "-v",
        ])
        .output() {
        if let Ok(out) = String::from_utf8(output.stdout) {
            let mut rules: Vec<&str> = out.split("\n").collect();
            if rules.len() >= 3 {
                rules = rules[2..].to_vec();
                let rules: Vec<IptablesRule> = rules
                    .iter()
                    .filter_map(|rule_str| {
                        parse_on_rule(rule_str, table, chain)
                    })
                    .collect();
                return rules;
            }
        }
    }
    return vec![];
}

pub fn iptabels_create_chain(table: &str, chain: &str) {
    let res = Command::new("iptables").args(vec![
        "-t",
        table,
        "-N",
        chain,
    ]).spawn();
    if let Ok(mut res) = res {
        let _ = res.wait();
    }
}

fn parse_on_rule(rule: &str, table: &str, chain: &str) -> Option<IptablesRule> {
    let segs = rule.split_ascii_whitespace().collect::<Vec<&str>>();
    if segs.len() < 9 {
        return None;
    }

    let iptables_rule = IptablesRule {
        table:      table.to_owned(),
        chain:      chain.to_owned(),
        in_:        segs[5].to_owned(),
        out:        segs[6].to_owned(),
        src:        segs[7].to_owned(),
        dst:        segs[8].to_owned(),
        target:     segs[2].to_owned(),
    };
    
    Some(iptables_rule)
}



pub fn iptables_append_rule(table: &str, chain: &str, rule_str: &str) {
    let mut rule;
    if table.len() == 0 {
        rule = vec![
            "-A",
            chain,
        ];
    }
    else {
        rule = vec![
            "-t",
            table,
            "-A",
            chain,
        ];
    }

    rule.append(&mut rule_str.split_ascii_whitespace().collect::<Vec<&str>>());

    let _ = Command::new("iptables").args(rule).spawn();
}


pub fn iptables_insert_rule(table: &str, chain: &str, rule_str: &str, position: i32) {
    let mut rule;
    if table.len() == 0 {
        rule = vec![
            "-I",
            chain,
        ];
    }
    else {
        rule = vec![
            "-t",
            table,
            "-I",
            chain,
        ];
    }

    if position != 0 {
        let position = format!("{}", position);
        rule.append(&mut vec!(&position));

        rule.append(&mut rule_str.split_ascii_whitespace().collect::<Vec<&str>>());

        let _ = Command::new("iptables").args(rule).spawn();
    }
    else {
        rule.append(&mut rule_str.split_ascii_whitespace().collect::<Vec<&str>>());
        let _ = Command::new("iptables").args(rule).spawn();
    }
}

pub fn iptables_delete_rule(table: &str, chain: &str, rule_str: &str) {
    let mut rule = vec![
        "-t",
        table,
        "-D",
        chain,
    ];
    rule.append(&mut rule_str.split_ascii_whitespace().collect::<Vec<&str>>());
    let _ = Command::new("iptables").args(rule).spawn();
}

pub fn iptables_find_rule(rule: &IptablesRule) -> bool {
    let rules = parse_iptables_table_chain(&rule.table, &rule.chain);
    rules.contains(rule)
}
