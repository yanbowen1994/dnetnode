use settings::Settings;
use file_tool::{file_exists, file_modify_time};
use std::path::Path;
use std::fs::metadata;

pub fn check_tinc_complete(tinc_home: &str) -> bool {
    if file_exists(&(tinc_home + "/tincd")) {
        true
    }
    false
}

pub fn check_pub_key(tinc_home: &str, pub_key_path: &str) -> bool {
    if let Some(sec) = file_modify_time(&(tinc_home.to_string() + pub_key_path)[..]) {
        if sec / 60 / 60 / 24 < 30 {
            return true;
        }
    }
    return false;


}
