#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub code:   u32,
    pub data:   Option<String>,
    pub msg:    Option<String>,
}
impl Response {
    pub fn succeed(msg: String) -> Self {
        Response {
            code:  200,
            data: None,
            msg: Some(msg),
        }
    }

    pub fn uid_failed() -> Self {
        Response {
            code:  401,
            data: None,
            msg:   Some("No authentication or authentication failure".to_string()),
        }
    }

    pub fn internal_error() -> Self {
        Self {
            code: 500,
            msg:  Some("".to_owned()),
            data: None,
        }
    }

    pub fn set_host_failed(ips: Vec<String>) -> Self {
        let data = match serde_json::to_string(&ips) {
            Ok(data) => data,
            Err(_) => "".to_owned(),
        };

        Response {
            code:  924,
            data:  Some(data),
            msg:   Some("Set host failed".to_string()),
        }
    }

    pub fn not_found() -> Self {
        Response {
            code:  404,
            data:  None,
            msg:   Some("Not Found".to_string()),
        }
    }
}
