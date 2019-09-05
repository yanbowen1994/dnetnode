use common_core::get_settings;
use tinc_plugin::PUB_KEY_FILENAME;
use std::time::SystemTime;
use std::path::Path;

pub fn check_pub_key() -> bool {
    let settings = get_settings();
    let pubkey_path = settings.tinc.home_path.to_owned() + PUB_KEY_FILENAME;
    let path = Path::new(&pubkey_path);
    if let Ok(fs) = std::fs::metadata(path) {
        if let Ok(time) = fs.modified() {
            if let Ok(now) = SystemTime::now().duration_since(time) {
                if now.as_secs() / 60 / 60 / 24 < 30 {
                    return true;
                }
            }
        }
    }
    return false;
}