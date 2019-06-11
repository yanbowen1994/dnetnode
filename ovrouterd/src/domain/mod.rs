//! 这个模块用于记录Ovrouter运行信息
//! GeoInfo：本机地理地址信息
//! ProxyInfo：公网ip， uuid等
//! TincInfo： 本机tinc运行参数
//! new() 创建空的结构体
//! load_local() 根据本地信息创建结构体，将会读取tinc公钥，ip，vip等
use std::io;

use serde_json;

use settings::Settings;

mod geo;
pub use self::geo::GeoInfo;
mod proxy;
pub use self::proxy::ProxyInfo;
pub use self::proxy::OnlineProxy;
mod tinc;
pub use self::tinc::TincInfo;
mod auth;
pub use self::auth::AuthInfo;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Load geo info error.")]
    GeoInfo(#[error(cause)] geo::Error),
    #[error(display = "Can not create log dir")]
    ProxyInfo(#[error(cause)] proxy::Error),
    #[error(display = "Can not create log dir")]
    Tinc(#[error(cause)] tinc::Error),
}

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

    pub fn new_from_local(settings: &Settings) -> Result<Self> {
        let mut geo_info = GeoInfo::new();
        let _ = geo_info.load_local(settings).map_err(Error::GeoInfo)?;
        let mut proxy_info = ProxyInfo::new();
        let _ = proxy_info.load_local().map_err(Error::ProxyInfo)?;
        // 使用geo ip 作为proxy ip, 而非使用本机路由default出口ip.
        proxy_info.proxy_ip = geo_info.ipaddr.clone();
        let mut tinc_info = TincInfo::new();
        let _ = tinc_info.load_local(
            &settings.tinc.home_path,
            &settings.tinc.pub_key_path
        )
            .map_err(Error::Tinc)?;

        debug!("geo_info: {:?}",geo_info);
        debug!("proxy_info: {:?}",proxy_info);
        debug!("tinc_info: {:?}",tinc_info);

        Ok(Info {
            geo_info,
            proxy_info,
            tinc_info,
        })
    }
}