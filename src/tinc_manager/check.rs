use settings::Settings;
use file_tool::File;
use std::path::Path;

pub fn check_tinc_complete(tinc_home: &str) -> bool {
    let file = File::new((tinc_home.to_string() + "/tincd").to_string());
    if file.file_exists() {
        return true;
    }
    false
}

pub fn check_pub_key(tinc_home: &str, pub_key_path: &str) -> bool {
    let file = File::new((tinc_home.to_string() + pub_key_path).to_string());
    if let Some(sec) = file.file_modify_time() {
        if sec / 60 / 60 / 24 < 30 {
            return true;
        }
    }
    return false;


}
