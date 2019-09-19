#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recv {
    pub code:        i32,
    pub msg:         Option<String>,
    pub data:        Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceId {
    pub deviceid: String,
}

impl DeviceId {
    pub fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}
