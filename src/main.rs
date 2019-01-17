extern crate serde_json;

use serde_json::{Value, Error};

pub mod net_tool;
pub mod sys_tool;
//pub mod proxy_info;
pub mod database;

fn main() {
//    let (res, _) = net_tool::fatch_url("http://52.25.79.82:10000/geoip_json.php");
//    println!("{:?}", res);
//    let json: Value = serde_json::from_str(&res).unwrap();
//    let city = json["city"].to_owned();
//    print!("{}", city);
    let db = database::Database::new();

    let a = String::from("123");
    let b = a.as_bytes();

    db.clear_range(&vec!["siteview"]);
//    db.clear(&vec!["name"]);
}