use std::string;

mod sys_tool;
pub mod net_tool;
pub mod file_tool;

use net_tool::{get_wan_name, get_local_ip};


fn main() {
    let res = get_wan_name().unwrap();
    print!("{:?}", res);
}