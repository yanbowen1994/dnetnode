#[derive(Serialize, Deserialize)]
pub enum HostStatusChange {
    TincUp,
    TincDown,
    HostUp(String),
    HostDown(String),
}