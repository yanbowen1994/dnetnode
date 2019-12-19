/// Results from fallible operations on the Tinc tunnel.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen when using the Tinc tunnel.
#[derive(err_derive::Error, Debug)]
#[allow(non_camel_case_types)]
pub enum Error {
    #[error(display = "Set route failed.")]
    SetRoute,

    #[error(display = "local_vip_not_init")]
    local_vip_not_init,

    #[error(display = "Tinc Info Proxy Ip Not Found")]
    TincInfo_connect_to_is_empty,

    #[error(display = "Tinc Info Proxy Ip Not Found")]
    IpNotFound,

    #[error(display = "Tinc Info Proxy Vip Not Found")]
    TincInfoProxyVipNotFound,

    #[error(display = "Get local ip before Rpc get_online_proxy")]
    GetLocalIpBeforeRpcGetOnlineProxy,

    /// Unable to start
    #[error(display = "duct can not start tinc")]
    NeverInitOperator,

    /// Unable to start
    #[error(display = "duct can not start tinc")]
    StartTincError,

    #[error(display = "duct can not start tinc")]
    AnotherTincRunning,

    /// Unable to stop
    #[error(display = "duct can not stop tinc")]
    StopTincError,

    /// tinc process not exist
    #[error(display = "tinc pidfile not exist")]
    PidfileNotExist,

    /// tinc process not exist
    #[error(display = "tinc process not exist")]
    TincNotExist,

    /// If should restart tinc, like config change, that error will skip to restart.
    #[error(display = "tinc process not start")]
    TincNeverStart,

    /// tinc host file not exist
    #[error(display = "tinc host file not exist")]
    FileNotExist(String),

    /// Failed create file
    #[error(display = "Failed create file")]
    FileCreateError(String),

    /// Tinc can't create key pair
    #[error(display = "Tinc can't create key pair")]
    CreatePubKeyError,

    /// Invalid tinc info
    #[error(display = "Invalid tinc info")]
    TincInfoError(String),

    /// Error while running "ip route".
    #[error(display = "Error while running \"ip route\"")]
    FailedToRunIpRoute(#[error(cause)] ::std::io::Error),

    /// Io error
    #[error(display = "Io error")]
    IoError(String),

    /// No wan dev
    #[error(display = "No wan dev")]
    NoWanDev,

    /// Address loaded from file is invalid
    #[error(display = "Address loaded from file is invalid")]
    ParseLocalIpError(#[error(cause)] std::net::AddrParseError),

    /// Address loaded from file is invalid
    #[error(display = "Address loaded from file is invalid")]
    ParseLocalVipError(#[error(cause)] std::net::AddrParseError),

    ///
    #[error(display = "Get default gateway error")]
    GetDefaultGatewayError(String),

    ///
    #[error(display = "Permissions error")]
    PermissionsError(#[error(cause)] std::io::Error),

    ///
    #[error(display = "Permissions error")]
    VnicNotFind(String),
}