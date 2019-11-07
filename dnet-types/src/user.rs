#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub user:       String,
    pub password:   String,
}