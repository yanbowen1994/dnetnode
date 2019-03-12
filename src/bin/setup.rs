extern crate ovrouter;
use ovrouter::settings::Settings;
use ovrouter::tinc_manager::check::*;
use ovrouter::tinc_manager::Tinc;
use ovrouter::domain::Info;
use ovrouter::http_server_client::Client;
use ovrouter::tinc_manager::install_tinc;

fn main() {
    let settings:Settings = Settings::load_config().expect("Error: can not parse settings.toml");
    // 初始化tinc操作
    let tinc = Tinc::new(settings.tinc.home_path.clone(), settings.tinc.pub_key_path.clone());
    // 监测tinc文件完整性，失败将安装tinc
    if !check_tinc_complete(&settings.tinc.home_path) {
        install_tinc(&settings, &tinc);
    }
}