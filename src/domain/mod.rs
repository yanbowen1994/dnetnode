use serde_json;
mod geo;
pub use self::geo::GeoInfo;
mod proxy;
pub use self::proxy::ProxyInfo;
mod tinc;
pub use self::tinc::TincInfo;

pub struct Info {
    geo_info: GeoInfo,
    proxy_info: ProxyInfo,
    tinc_info: TincInfo,
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
}