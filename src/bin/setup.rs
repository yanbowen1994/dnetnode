extern crate ovrouter;
use ovrouter::tinc_manager::install_tinc::install_tinc;
use ovrouter::settings::Settings;
use ovrouter::tinc_manager::Operater;

fn main() {
    let settings = Settings::load_config().expect("Can't parse settings.toml");
    let operater = Operater::new(&settings.tinc.home_path, &settings.tinc.pub_key_path);
    install_tinc(&settings, &operater);
    println!("Done");
}
