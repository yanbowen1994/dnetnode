//! 这个模块用于记录Ovrouter运行信息
//! GeoInfo：本机地理地址信息
//! ProxyInfo：公网ip， uuid等
//! TincInfo： 本机tinc运行参数
//! new() 创建空的结构体
//! load_local() 根据本地信息创建结构体，将会读取tinc公钥，ip，vip等
use std::sync::mpsc;

use common_core::traits::InfoTrait;

mod proxy;
pub use self::proxy::ProxyInfo;
pub use self::proxy::OnlineProxy;
mod tinc;
pub use self::tinc::TincInfo;
mod auth;
pub use self::auth::AuthInfo;
use common_core::daemon::DaemonEvent;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Can not create log dir")]
    ProxyInfo(#[error(cause)] proxy::Error),
    #[error(display = "Can not create log dir")]
    Tinc(#[error(cause)] tinc::Error),
}

#[derive(Debug, Clone)]
pub struct Info {
    pub proxy_info: ProxyInfo,
    pub tinc_info: TincInfo,
}
impl InfoTrait for Info {
    fn new(daemon_event_tx: mpsc::Sender<DaemonEvent>) -> Self {
        let mut proxy_info = ProxyInfo::new();
        let _ = proxy_info.load_local();

        let mut tinc_info = TincInfo::new();
        tinc_info.load_local();

        debug!("proxy_info: {:?}", proxy_info);
        debug!("tinc_info: {:?}", tinc_info);

        Info {
            proxy_info,
            tinc_info,
        }
    }

    fn create_uid(&mut self) {
        self.proxy_info.create_uid()
    }
}