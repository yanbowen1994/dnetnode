extern crate setup;
use setup::settings::Settings;
use setup::install_tinc;

fn main() {
    let settings:Settings = Settings::load_config().expect("Error: can not parse settings.toml");
    install_tinc(&settings);
}