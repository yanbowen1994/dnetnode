#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub name:       String,
    pub password:   String,
}

impl User {
    pub fn new(user_name: &str, password: &str) -> Self{
        Self {
            name: user_name.to_owned(),
            password: password.to_owned(),
        }
    }

    pub fn to_json_str(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}