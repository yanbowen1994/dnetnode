#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub token: String,
}

impl NodeInfo {
    pub fn new() -> Self {
        Self {
            token: String::new(),
        }
    }
}