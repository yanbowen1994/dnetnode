use std::net::AddrParseError;
use tinc_plugin::TincOperatorError;

pub type Result<T> = std::result::Result<T, Error>;

#[allow(non_camel_case_types)]
#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Parse Ip failed.")]
    ParseIp(#[error(cause)] AddrParseError),

    #[error(display = "search_team_by_mac failed.")]
    search_team_by_mac,

    #[error(display = "Login can not parse json str.")]
    ParseJsonStr(#[error(cause)] serde_json::Error),

    #[error(display = "Fail to DNS parse conductor cluster domain.")]
    ParseConductorDomain(String),

    #[error(display = "Login can not parse json str.")]
    LoginParseJsonStr(#[error(cause)] serde_json::Error),

    #[error(display = "Login failed no cookie back.")]
    LoginResNoCookie,

    #[error(display = "Login failed.")]
    LoginFailed(String),

    #[error(display = "Proxy register failed.")]
    RegisterFailed(String),

    #[error(display = "Login can not parse json str.")]
    RegisterParseJsonStr(#[error(cause)] serde_json::Error),

    #[error(display = "Get online client failed.")]
    GetOnlineProxy(String),

    #[error(display = "Login can not parse json str.")]
    GetOnlineProxyParseJsonStr(#[error(cause)] serde_json::Error),

    #[error(display = "client_get_online_client - get online client data invalid.")]
    GetOnlineProxyInvalidData,

    #[error(display = "Heartbeat can not parse json str.")]
    HeartbeatJsonStr(#[error(cause)] serde_json::Error),

    #[error(display = "Heartbeat timeout.")]
    HeartbeatTimeout,

    #[error(display = "Heartbeat failed.")]
    HeartbeatFailed,

    #[error(display = "reqwest::Error.")]
    Reqwest(#[error(cause)] reqwest::Error),

    #[error(display = "operator::Error.")]
    TincOperator(#[error(cause)] TincOperatorError),
}