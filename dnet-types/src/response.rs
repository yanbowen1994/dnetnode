pub use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    pub code:   u32,
    pub msg:    String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data:   Option<Value>
}

impl Response {
    pub fn set_msg(mut self, msg: String) -> Self {
        self.msg = msg;
        self
    }

    pub fn set_data(mut self, data: Option<Value>) -> Self {
        self.data = data;
        self
    }

    pub fn success() -> Self {
        Self {
            code: 200,
            msg:  "".to_owned(),
            data: None,
        }
    }

    pub fn internal_error() -> Self {
        Self {
            code: 500,
            msg:  "".to_owned(),
            data: None,
        }
    }

    pub fn exec_timeout() -> Self {
        Self {
            code: 500,
            msg:  "Timeout".to_string(),
            data: None,
        }
    }
}