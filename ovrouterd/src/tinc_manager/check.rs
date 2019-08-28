use file_tool::File;
use settings::get_settings;
use tinc_plugin::PUB_KEY_FILENAME;

pub fn check_tinc_complete(tinc_home: &str) -> bool {
    let file = File::new((tinc_home.to_string() + "/tincd").to_string());
    if file.file_exists() {
        return true;
    }
    false
}

pub fn check_pub_key() -> bool {
    let settings = get_settings();
    let tinc_home = settings.tinc.home_path.to_owned();
    let file = File::new((tinc_home + PUB_KEY_FILENAME).to_string());
    if let Some(sec) = file.file_modify_time() {
        if sec / 60 / 60 / 24 < 30 {
            return true;
        }
    }
    return false;
}