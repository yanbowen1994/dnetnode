//! 这个模块用于记录Ovrouter运行信息
//! GeoInfo：本机地理地址信息
//! ProxyInfo：公网ip， uuid等
//! TincInfo： 本机tinc运行参数
//! new() 创建空的结构体
//! load_local() 根据本地信息创建结构体，将会读取tinc公钥，ip，vip等

use serde_json;

use settings::Settings;

mod geo;
pub use self::geo::GeoInfo;
mod proxy;
pub use self::proxy::ProxyInfo;
mod tinc;
pub use self::tinc::TincInfo;

#[derive(Debug, Clone)]
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
        let mut geo_info = GeoInfo::new();
        if !geo_info.load_local(settings) {
            error!("Load local proxy info error");
        };
        let mut proxy_info = ProxyInfo::new();
        if !proxy_info.load_local() {
            error!("Load local proxy info error");
        }
        let mut tinc_info = TincInfo::new();
        if !tinc_info.load_local(&settings.tinc.home_path, &settings.tinc.pub_key_path) {
            error!("Load local tinc info error");
        }
        Info {
            geo_info,
            proxy_info,
            tinc_info,
        }
    }
}