use serde_json;

use settings::Settings;

mod geo;
pub use self::geo::GeoInfo;
mod proxy;
pub use self::proxy::ProxyInfo;
mod tinc;
pub use self::tinc::TincInfo;

pub struct Info {
    pub geo_info: GeoInfo,
    pub proxy_info: ProxyInfo,
    pub tinc_info: TincInfo,
}
impl Info {
    pub fn new() -> Self {
        let geo_info = GeoInfo::new();
        let proxy_info = ProxyInfo::new();
        let tinc_info = TincInfo::new();
        Info {
            geo_info,
            proxy_info,
            tinc_info,
        }
    }

    pub fn new_from_local(settings: &Settings) -> Self {
        let geo_info = GeoInfo::new();
        let mut proxy_info = ProxyInfo::new();
        if proxy_info.load_local() {
            debug!("Load local proxy info error");
        }
        let mut tinc_info = TincInfo::new();
        if tinc_info.load_local(&settings.tinc.home_path, &settings.tinc.pub_key_path) {
            debug!("Load local tinc info error");
        }
        Info {
            geo_info,
            proxy_info,
            tinc_info,
        }
    }
}