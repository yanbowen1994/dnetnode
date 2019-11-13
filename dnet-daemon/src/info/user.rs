use crate::settings::get_settings;

#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub name:              Option<String>,
    pub email:             Option<String>,
    pub photo:             Option<String>,
}

impl UserInfo {
    pub fn new() -> Self {
        let name = get_settings().common.username.clone();
        let name = if name.len() != 0 {
            Some(name)
        }
        else {
            None
        };
        Self {
            name,
            email:          None,
            photo:          None,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(&self).unwrap()
    }
}